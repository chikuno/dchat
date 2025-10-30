variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "us-east-1"
}

variable "environment" {
  description = "Environment name (staging, production)"
  type        = string
  validation {
    condition     = contains(["staging", "production"], var.environment)
    error_message = "Environment must be staging or production."
  }
}

variable "domain_name" {
  description = "Domain name for the application"
  type        = string
  default     = "dchat.network"
}

variable "vpc_cidr" {
  description = "CIDR block for VPC"
  type        = string
  default     = "10.0.0.0/16"
}

variable "private_subnet_cidrs" {
  description = "CIDR blocks for private subnets"
  type        = list(string)
  default     = ["10.0.1.0/24", "10.0.2.0/24", "10.0.3.0/24"]
}

variable "public_subnet_cidrs" {
  description = "CIDR blocks for public subnets"
  type        = list(string)
  default     = ["10.0.101.0/24", "10.0.102.0/24", "10.0.103.0/24"]
}

variable "kubernetes_version" {
  description = "Kubernetes version"
  type        = string
  default     = "1.28"
}

variable "relay_node_instance_types" {
  description = "Instance types for relay nodes"
  type        = list(string)
  default     = ["c6i.xlarge"]
}

variable "relay_node_min_size" {
  description = "Minimum number of relay nodes"
  type        = number
  default     = 3
}

variable "relay_node_max_size" {
  description = "Maximum number of relay nodes"
  type        = number
  default     = 10
}

variable "relay_node_desired_size" {
  description = "Desired number of relay nodes"
  type        = number
  default     = 5
}

variable "app_node_instance_types" {
  description = "Instance types for application nodes"
  type        = list(string)
  default     = ["t3.large"]
}

variable "app_node_min_size" {
  description = "Minimum number of application nodes"
  type        = number
  default     = 2
}

variable "app_node_max_size" {
  description = "Maximum number of application nodes"
  type        = number
  default     = 10
}

variable "app_node_desired_size" {
  description = "Desired number of application nodes"
  type        = number
  default     = 3
}

variable "db_instance_class" {
  description = "RDS instance class"
  type        = string
  default     = "db.t3.large"
}

variable "db_allocated_storage" {
  description = "Initial database storage in GB"
  type        = number
  default     = 100
}

variable "db_max_allocated_storage" {
  description = "Maximum database storage in GB (autoscaling)"
  type        = number
  default     = 500
}

variable "db_name" {
  description = "Database name"
  type        = string
  default     = "dchat"
}

variable "db_username" {
  description = "Database master username"
  type        = string
  default     = "dchat"
  sensitive   = true
}

variable "backup_retention_days" {
  description = "Number of days to retain backups"
  type        = number
  default     = 90
}

variable "log_retention_days" {
  description = "CloudWatch log retention in days"
  type        = number
  default     = 30
}
