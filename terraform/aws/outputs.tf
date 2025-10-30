output "cluster_name" {
  description = "EKS cluster name"
  value       = module.eks.cluster_name
}

output "cluster_endpoint" {
  description = "EKS cluster endpoint"
  value       = module.eks.cluster_endpoint
}

output "cluster_security_group_id" {
  description = "EKS cluster security group ID"
  value       = module.eks.cluster_security_group_id
}

output "cluster_certificate_authority_data" {
  description = "Base64 encoded certificate data"
  value       = module.eks.cluster_certificate_authority_data
  sensitive   = true
}

output "database_endpoint" {
  description = "RDS database endpoint"
  value       = module.db.db_instance_endpoint
}

output "database_name" {
  description = "Database name"
  value       = module.db.db_instance_name
}

output "alb_dns_name" {
  description = "ALB DNS name"
  value       = module.alb.lb_dns_name
}

output "alb_zone_id" {
  description = "ALB hosted zone ID"
  value       = module.alb.lb_zone_id
}

output "backup_bucket" {
  description = "S3 backup bucket name"
  value       = aws_s3_bucket.backups.id
}

output "kms_key_id" {
  description = "KMS key ID for secrets encryption"
  value       = aws_kms_key.secrets.id
}

output "vpc_id" {
  description = "VPC ID"
  value       = module.vpc.vpc_id
}

output "private_subnet_ids" {
  description = "Private subnet IDs"
  value       = module.vpc.private_subnets
}

output "public_subnet_ids" {
  description = "Public subnet IDs"
  value       = module.vpc.public_subnets
}

output "configure_kubectl" {
  description = "Command to configure kubectl"
  value       = "aws eks update-kubeconfig --name ${module.eks.cluster_name} --region ${var.aws_region}"
}
