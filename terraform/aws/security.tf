# Security Hardening Configuration for dchat Production

## HSM Integration (AWS KMS / CloudHSM)

resource "aws_kms_key" "validator_signing_key" {
  description             = "dchat validator signing key in HSM"
  deletion_window_in_days = 30
  key_usage               = "SIGN_VERIFY"
  customer_master_key_spec = "ECC_SECG_P256K1"
  
  enable_key_rotation = true
  
  tags = {
    Name        = "dchat-validator-signing-key"
    Environment = var.environment
    Purpose     = "validator-signing"
  }
}

resource "aws_kms_alias" "validator_signing_key" {
  name          = "alias/dchat-validator-${var.environment}"
  target_key_id = aws_kms_key.validator_signing_key.key_id
}

# CloudHSM cluster for enhanced security (optional, higher cost)
resource "aws_cloudhsm_v2_cluster" "dchat_hsm" {
  count = var.enable_cloudhsm ? 1 : 0
  
  hsm_type   = "hsm1.medium"
  subnet_ids = module.vpc.private_subnets
  
  tags = {
    Name        = "dchat-hsm-${var.environment}"
    Environment = var.environment
  }
}

resource "aws_cloudhsm_v2_hsm" "dchat_hsm_instance" {
  count      = var.enable_cloudhsm ? var.hsm_instance_count : 0
  cluster_id = aws_cloudhsm_v2_cluster.dchat_hsm[0].id
  subnet_id  = element(module.vpc.private_subnets, count.index)
  
  tags = {
    Name = "dchat-hsm-instance-${count.index}"
  }
}

## DDoS Protection (AWS Shield Advanced + CloudFront)

resource "aws_shield_protection" "alb" {
  count        = var.enable_shield_advanced ? 1 : 0
  name         = "dchat-alb-protection"
  resource_arn = module.alb.lb_arn
  
  tags = {
    Environment = var.environment
  }
}

resource "aws_cloudfront_distribution" "dchat_cdn" {
  count   = var.enable_cloudfront ? 1 : 0
  enabled = true
  
  origin {
    domain_name = module.alb.lb_dns_name
    origin_id   = "dchat-alb"
    
    custom_origin_config {
      http_port              = 80
      https_port             = 443
      origin_protocol_policy = "https-only"
      origin_ssl_protocols   = ["TLSv1.2"]
    }
  }
  
  default_cache_behavior {
    allowed_methods  = ["DELETE", "GET", "HEAD", "OPTIONS", "PATCH", "POST", "PUT"]
    cached_methods   = ["GET", "HEAD"]
    target_origin_id = "dchat-alb"
    
    forwarded_values {
      query_string = true
      headers      = ["Authorization", "Host"]
      
      cookies {
        forward = "none"
      }
    }
    
    viewer_protocol_policy = "redirect-to-https"
    min_ttl                = 0
    default_ttl            = 3600
    max_ttl                = 86400
    compress               = true
  }
  
  restrictions {
    geo_restriction {
      restriction_type = "none"
    }
  }
  
  viewer_certificate {
    acm_certificate_arn      = var.certificate_arn
    ssl_support_method       = "sni-only"
    minimum_protocol_version = "TLSv1.2_2021"
  }
  
  web_acl_id = var.enable_waf ? aws_wafv2_web_acl.dchat[0].arn : null
  
  tags = {
    Name        = "dchat-cdn-${var.environment}"
    Environment = var.environment
  }
}

## WAF (Web Application Firewall)

resource "aws_wafv2_web_acl" "dchat" {
  count = var.enable_waf ? 1 : 0
  
  name  = "dchat-waf-${var.environment}"
  scope = "REGIONAL"
  
  default_action {
    allow {}
  }
  
  # Rate limiting rule
  rule {
    name     = "rate-limit"
    priority = 1
    
    action {
      block {}
    }
    
    statement {
      rate_based_statement {
        limit              = 2000
        aggregate_key_type = "IP"
      }
    }
    
    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "RateLimitRule"
      sampled_requests_enabled   = true
    }
  }
  
  # AWS Managed Rules - Core Rule Set
  rule {
    name     = "aws-managed-rules-core"
    priority = 2
    
    override_action {
      none {}
    }
    
    statement {
      managed_rule_group_statement {
        name        = "AWSManagedRulesCommonRuleSet"
        vendor_name = "AWS"
      }
    }
    
    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "AWSManagedRulesCore"
      sampled_requests_enabled   = true
    }
  }
  
  # Known bad inputs
  rule {
    name     = "aws-managed-rules-known-bad-inputs"
    priority = 3
    
    override_action {
      none {}
    }
    
    statement {
      managed_rule_group_statement {
        name        = "AWSManagedRulesKnownBadInputsRuleSet"
        vendor_name = "AWS"
      }
    }
    
    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "AWSManagedRulesKnownBadInputs"
      sampled_requests_enabled   = true
    }
  }
  
  # SQL injection protection
  rule {
    name     = "sql-injection-protection"
    priority = 4
    
    override_action {
      none {}
    }
    
    statement {
      managed_rule_group_statement {
        name        = "AWSManagedRulesSQLiRuleSet"
        vendor_name = "AWS"
      }
    }
    
    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "SQLInjectionProtection"
      sampled_requests_enabled   = true
    }
  }
  
  visibility_config {
    cloudwatch_metrics_enabled = true
    metric_name                = "DchatWAF"
    sampled_requests_enabled   = true
  }
  
  tags = {
    Name        = "dchat-waf-${var.environment}"
    Environment = var.environment
  }
}

## Security Groups - Enhanced Rules

resource "aws_security_group_rule" "eks_ingress_vpn_only" {
  count = var.restrict_eks_access ? 1 : 0
  
  type              = "ingress"
  from_port         = 443
  to_port           = 443
  protocol          = "tcp"
  cidr_blocks       = [var.vpn_cidr_block]
  security_group_id = module.eks.cluster_security_group_id
  description       = "Allow EKS API access from VPN only"
}

## Secrets Manager for Sensitive Configuration

resource "aws_secretsmanager_secret" "database_credentials" {
  name = "dchat/database/${var.environment}"
  
  rotation_rules {
    automatically_after_days = 90
  }
  
  tags = {
    Environment = var.environment
  }
}

resource "aws_secretsmanager_secret_version" "database_credentials" {
  secret_id = aws_secretsmanager_secret.database_credentials.id
  
  secret_string = jsonencode({
    username = var.database_username
    password = var.database_password
    host     = module.rds.db_instance_address
    port     = module.rds.db_instance_port
    database = var.database_name
  })
}

resource "aws_secretsmanager_secret" "api_keys" {
  name = "dchat/api-keys/${var.environment}"
  
  tags = {
    Environment = var.environment
  }
}

## GuardDuty (Threat Detection)

resource "aws_guardduty_detector" "main" {
  enable = true
  
  finding_publishing_frequency = "FIFTEEN_MINUTES"
  
  datasources {
    s3_logs {
      enable = true
    }
    kubernetes {
      audit_logs {
        enable = true
      }
    }
  }
  
  tags = {
    Environment = var.environment
  }
}

## Security Hub (Compliance Monitoring)

resource "aws_securityhub_account" "main" {}

resource "aws_securityhub_standards_subscription" "cis" {
  standards_arn = "arn:aws:securityhub:::ruleset/cis-aws-foundations-benchmark/v/1.2.0"
  depends_on    = [aws_securityhub_account.main]
}

resource "aws_securityhub_standards_subscription" "pci_dss" {
  standards_arn = "arn:aws:securityhub:${var.aws_region}::standards/pci-dss/v/3.2.1"
  depends_on    = [aws_securityhub_account.main]
}

## Config (Configuration Compliance)

resource "aws_config_configuration_recorder" "main" {
  name     = "dchat-config-recorder"
  role_arn = aws_iam_role.config.arn
  
  recording_group {
    all_supported                 = true
    include_global_resource_types = true
  }
}

resource "aws_config_delivery_channel" "main" {
  name           = "dchat-config-delivery"
  s3_bucket_name = aws_s3_bucket.config_logs.id
  
  depends_on = [aws_config_configuration_recorder.main]
}

resource "aws_s3_bucket" "config_logs" {
  bucket = "dchat-config-logs-${var.environment}-${data.aws_caller_identity.current.account_id}"
  
  tags = {
    Name        = "dchat-config-logs"
    Environment = var.environment
  }
}

resource "aws_s3_bucket_versioning" "config_logs" {
  bucket = aws_s3_bucket.config_logs.id
  
  versioning_configuration {
    status = "Enabled"
  }
}

## IAM Roles for Security Services

resource "aws_iam_role" "config" {
  name = "dchat-config-role-${var.environment}"
  
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "config.amazonaws.com"
        }
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "config" {
  role       = aws_iam_role.config.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/ConfigRole"
}

## Data source for current AWS account

data "aws_caller_identity" "current" {}

## Variables for security configuration

variable "enable_cloudhsm" {
  description = "Enable CloudHSM for enhanced key security"
  type        = bool
  default     = false
}

variable "hsm_instance_count" {
  description = "Number of HSM instances (minimum 2 for HA)"
  type        = number
  default     = 2
}

variable "enable_shield_advanced" {
  description = "Enable AWS Shield Advanced (additional cost)"
  type        = bool
  default     = true
}

variable "enable_cloudfront" {
  description = "Enable CloudFront CDN for DDoS protection"
  type        = bool
  default     = true
}

variable "enable_waf" {
  description = "Enable WAF for application-layer protection"
  type        = bool
  default     = true
}

variable "restrict_eks_access" {
  description = "Restrict EKS API access to VPN only"
  type        = bool
  default     = true
}

variable "vpn_cidr_block" {
  description = "CIDR block for VPN access"
  type        = string
  default     = "10.100.0.0/16"
}
