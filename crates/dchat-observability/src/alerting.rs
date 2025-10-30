use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

/// Alert severity level
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Alert state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertState {
    /// Alert is firing
    Firing,
    /// Alert condition resolved
    Resolved,
    /// Alert is silenced
    Silenced,
}

/// Comparison operator for alert rules
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operator {
    GreaterThan,
    LessThan,
    Equal,
    NotEqual,
    GreaterOrEqual,
    LessOrEqual,
}

/// Alert rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub metric_name: String,
    pub operator: Operator,
    pub threshold: f64,
    pub duration_secs: u64, // Alert fires if condition true for this duration
    pub severity: Severity,
    pub enabled: bool,
}

impl AlertRule {
    /// Create a new alert rule
    pub fn new(
        name: String,
        description: String,
        metric_name: String,
        operator: Operator,
        threshold: f64,
        duration_secs: u64,
        severity: Severity,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            metric_name,
            operator,
            threshold,
            duration_secs,
            severity,
            enabled: true,
        }
    }

    /// Evaluate rule against a metric value
    pub fn evaluate(&self, value: f64) -> bool {
        if !self.enabled {
            return false;
        }

        match self.operator {
            Operator::GreaterThan => value > self.threshold,
            Operator::LessThan => value < self.threshold,
            Operator::Equal => (value - self.threshold).abs() < f64::EPSILON,
            Operator::NotEqual => (value - self.threshold).abs() >= f64::EPSILON,
            Operator::GreaterOrEqual => value >= self.threshold,
            Operator::LessOrEqual => value <= self.threshold,
        }
    }
}

/// Alert instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub state: AlertState,
    pub severity: Severity,
    pub message: String,
    pub labels: HashMap<String, String>,
    pub fired_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub silenced_until: Option<DateTime<Utc>>,
}

/// Notification channel type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelType {
    Email,
    Slack,
    PagerDuty,
    Webhook,
    Console,
}

/// Notification channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    pub id: Uuid,
    pub name: String,
    pub channel_type: ChannelType,
    pub config: HashMap<String, String>, // email address, webhook URL, etc.
    pub enabled: bool,
}

/// Escalation policy level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationLevel {
    pub level: u32,
    pub channels: Vec<Uuid>, // Channel IDs
    pub delay_secs: u64,     // Delay before escalating to this level
}

/// Escalation policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationPolicy {
    pub id: Uuid,
    pub name: String,
    pub levels: Vec<EscalationLevel>,
}

impl EscalationPolicy {
    /// Create a new escalation policy
    pub fn new(name: String, levels: Vec<EscalationLevel>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            levels,
        }
    }

    /// Get channels for a given severity and time elapsed
    pub fn get_channels(&self, elapsed_secs: u64) -> Vec<Uuid> {
        let mut channels = Vec::new();

        for level in &self.levels {
            if elapsed_secs >= level.delay_secs {
                channels.extend(level.channels.iter());
            }
        }

        channels
    }
}

/// Alert routing rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    pub id: Uuid,
    pub severity_filter: Option<Severity>, // None = all severities
    pub label_filter: HashMap<String, String>, // Must match all labels
    pub escalation_policy_id: Uuid,
}

/// Alert manager for rule evaluation and notification
pub struct AlertManager {
    rules: Arc<RwLock<HashMap<Uuid, AlertRule>>>,
    alerts: Arc<RwLock<HashMap<Uuid, Alert>>>,
    channels: Arc<RwLock<HashMap<Uuid, NotificationChannel>>>,
    escalation_policies: Arc<RwLock<HashMap<Uuid, EscalationPolicy>>>,
    routing_rules: Arc<RwLock<Vec<RoutingRule>>>,
    rule_state: Arc<RwLock<HashMap<Uuid, (bool, DateTime<Utc>)>>>, // (condition_met, since_when)
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new() -> Self {
        Self {
            rules: Arc::new(RwLock::new(HashMap::new())),
            alerts: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
            escalation_policies: Arc::new(RwLock::new(HashMap::new())),
            routing_rules: Arc::new(RwLock::new(Vec::new())),
            rule_state: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add an alert rule
    pub fn add_rule(&self, rule: AlertRule) -> Uuid {
        let id = rule.id;
        let mut rules = self.rules.write().unwrap();
        rules.insert(id, rule);
        id
    }

    /// Remove an alert rule
    pub fn remove_rule(&self, rule_id: Uuid) {
        let mut rules = self.rules.write().unwrap();
        rules.remove(&rule_id);
    }

    /// Add a notification channel
    pub fn add_channel(&self, channel: NotificationChannel) -> Uuid {
        let id = channel.id;
        let mut channels = self.channels.write().unwrap();
        channels.insert(id, channel);
        id
    }

    /// Add an escalation policy
    pub fn add_escalation_policy(&self, policy: EscalationPolicy) -> Uuid {
        let id = policy.id;
        let mut policies = self.escalation_policies.write().unwrap();
        policies.insert(id, policy);
        id
    }

    /// Add a routing rule
    pub fn add_routing_rule(&self, rule: RoutingRule) {
        let mut routing = self.routing_rules.write().unwrap();
        routing.push(rule);
    }

    /// Evaluate a metric against all rules
    pub fn evaluate_metric(&self, metric_name: &str, value: f64) -> Vec<Uuid> {
        let mut fired_alerts = Vec::new();
        let rules = self.rules.read().unwrap();
        let now = Utc::now();

        for rule in rules.values() {
            if rule.metric_name != metric_name {
                continue;
            }

            let condition_met = rule.evaluate(value);
            let mut state = self.rule_state.write().unwrap();

            if condition_met {
                // Check if condition has been met for required duration
                let entry = state.entry(rule.id).or_insert((true, now));

                if entry.0 {
                    let elapsed = now.signed_duration_since(entry.1);
                    if elapsed.num_seconds() >= rule.duration_secs as i64 {
                        // Fire alert
                        let alert_id = self.fire_alert(rule, value);
                        fired_alerts.push(alert_id);
                        // Reset state
                        *entry = (false, now);
                    }
                } else {
                    // Start tracking
                    *entry = (true, now);
                }
            } else {
                // Condition not met, reset state
                if let Some(entry) = state.get_mut(&rule.id) {
                    *entry = (false, now);
                }
            }
        }

        fired_alerts
    }

    /// Fire an alert
    fn fire_alert(&self, rule: &AlertRule, value: f64) -> Uuid {
        let alert = Alert {
            id: Uuid::new_v4(),
            rule_id: rule.id,
            state: AlertState::Firing,
            severity: rule.severity.clone(),
            message: format!(
                "{}: {} {} {} (current: {})",
                rule.name,
                rule.metric_name,
                match rule.operator {
                    Operator::GreaterThan => ">",
                    Operator::LessThan => "<",
                    Operator::Equal => "==",
                    Operator::NotEqual => "!=",
                    Operator::GreaterOrEqual => ">=",
                    Operator::LessOrEqual => "<=",
                },
                rule.threshold,
                value
            ),
            labels: HashMap::new(),
            fired_at: Utc::now(),
            resolved_at: None,
            silenced_until: None,
        };

        let alert_id = alert.id;
        let mut alerts = self.alerts.write().unwrap();
        alerts.insert(alert_id, alert);
        alert_id
    }

    /// Resolve an alert
    pub fn resolve_alert(&self, alert_id: Uuid) {
        let mut alerts = self.alerts.write().unwrap();
        if let Some(alert) = alerts.get_mut(&alert_id) {
            alert.state = AlertState::Resolved;
            alert.resolved_at = Some(Utc::now());
        }
    }

    /// Silence an alert for a duration
    pub fn silence_alert(&self, alert_id: Uuid, duration_secs: u64) {
        let mut alerts = self.alerts.write().unwrap();
        if let Some(alert) = alerts.get_mut(&alert_id) {
            alert.state = AlertState::Silenced;
            alert.silenced_until = Some(Utc::now() + Duration::seconds(duration_secs as i64));
        }
    }

    /// Get all active (firing) alerts
    pub fn get_active_alerts(&self) -> Vec<Alert> {
        let alerts = self.alerts.read().unwrap();
        alerts
            .values()
            .filter(|a| a.state == AlertState::Firing)
            .cloned()
            .collect()
    }

    /// Get alerts by severity
    pub fn get_alerts_by_severity(&self, severity: &Severity) -> Vec<Alert> {
        let alerts = self.alerts.read().unwrap();
        alerts
            .values()
            .filter(|a| &a.severity == severity)
            .cloned()
            .collect()
    }

    /// Route alert to appropriate channels based on routing rules
    pub fn route_alert(&self, alert_id: Uuid) -> Vec<Uuid> {
        let alerts = self.alerts.read().unwrap();
        let alert = match alerts.get(&alert_id) {
            Some(a) => a,
            None => return Vec::new(),
        };

        let routing = self.routing_rules.read().unwrap();
        let policies = self.escalation_policies.read().unwrap();

        for rule in routing.iter() {
            // Check severity filter
            if let Some(ref severity_filter) = rule.severity_filter {
                if severity_filter != &alert.severity {
                    continue;
                }
            }

            // Check label filter
            let labels_match = rule
                .label_filter
                .iter()
                .all(|(k, v)| alert.labels.get(k) == Some(v));

            if !labels_match {
                continue;
            }

            // Get escalation policy
            if let Some(policy) = policies.get(&rule.escalation_policy_id) {
                let elapsed = Utc::now()
                    .signed_duration_since(alert.fired_at)
                    .num_seconds() as u64;
                return policy.get_channels(elapsed);
            }
        }

        Vec::new()
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_rule_evaluation() {
        let rule = AlertRule::new(
            "High CPU".to_string(),
            "CPU > 80%".to_string(),
            "cpu_usage".to_string(),
            Operator::GreaterThan,
            80.0,
            60,
            Severity::Warning,
        );

        assert!(rule.evaluate(85.0));
        assert!(!rule.evaluate(75.0));
    }

    #[test]
    fn test_alert_rule_operators() {
        let mut rule = AlertRule::new(
            "Test".to_string(),
            "Test".to_string(),
            "metric".to_string(),
            Operator::LessThan,
            10.0,
            0,
            Severity::Info,
        );

        rule.operator = Operator::LessThan;
        assert!(rule.evaluate(5.0));
        assert!(!rule.evaluate(15.0));

        rule.operator = Operator::Equal;
        assert!(rule.evaluate(10.0));
        assert!(!rule.evaluate(10.1));

        rule.operator = Operator::GreaterOrEqual;
        assert!(rule.evaluate(10.0));
        assert!(rule.evaluate(11.0));
        assert!(!rule.evaluate(9.0));
    }

    #[test]
    fn test_alert_manager_add_rule() {
        let manager = AlertManager::new();
        let rule = AlertRule::new(
            "Test Rule".to_string(),
            "Description".to_string(),
            "test_metric".to_string(),
            Operator::GreaterThan,
            100.0,
            60,
            Severity::Critical,
        );

        let rule_id = manager.add_rule(rule);
        assert_ne!(rule_id, Uuid::nil());
    }

    #[test]
    fn test_evaluate_metric() {
        let manager = AlertManager::new();
        let rule = AlertRule::new(
            "High Error Rate".to_string(),
            "Error rate > 5%".to_string(),
            "error_rate".to_string(),
            Operator::GreaterThan,
            5.0,
            0, // Fire immediately
            Severity::Error,
        );

        manager.add_rule(rule);

        // First evaluation - condition met
        let fired = manager.evaluate_metric("error_rate", 10.0);
        assert_eq!(fired.len(), 1);

        // Check alert was created
        let active = manager.get_active_alerts();
        assert_eq!(active.len(), 1);
    }

    #[test]
    fn test_resolve_alert() {
        let manager = AlertManager::new();
        let rule = AlertRule::new(
            "Test".to_string(),
            "Test".to_string(),
            "metric".to_string(),
            Operator::GreaterThan,
            50.0,
            0,
            Severity::Warning,
        );

        manager.add_rule(rule);
        let fired = manager.evaluate_metric("metric", 100.0);
        let alert_id = fired[0];

        manager.resolve_alert(alert_id);

        let active = manager.get_active_alerts();
        assert_eq!(active.len(), 0);
    }

    #[test]
    fn test_silence_alert() {
        let manager = AlertManager::new();
        let rule = AlertRule::new(
            "Test".to_string(),
            "Test".to_string(),
            "metric".to_string(),
            Operator::GreaterThan,
            50.0,
            0,
            Severity::Info,
        );

        manager.add_rule(rule);
        let fired = manager.evaluate_metric("metric", 100.0);
        let alert_id = fired[0];

        manager.silence_alert(alert_id, 3600);

        let active = manager.get_active_alerts();
        assert_eq!(active.len(), 0); // Silenced alerts are not active
    }

    #[test]
    fn test_escalation_policy() {
        let level1 = EscalationLevel {
            level: 1,
            channels: vec![Uuid::new_v4()],
            delay_secs: 0,
        };

        let level2 = EscalationLevel {
            level: 2,
            channels: vec![Uuid::new_v4()],
            delay_secs: 300, // 5 minutes
        };

        let policy = EscalationPolicy::new("Test Policy".to_string(), vec![level1, level2]);

        // Immediately after alert
        let channels = policy.get_channels(0);
        assert_eq!(channels.len(), 1);

        // After 5 minutes
        let channels = policy.get_channels(300);
        assert_eq!(channels.len(), 2);
    }

    #[test]
    fn test_routing_rule() {
        let manager = AlertManager::new();

        // Create channel
        let channel = NotificationChannel {
            id: Uuid::new_v4(),
            name: "Slack".to_string(),
            channel_type: ChannelType::Slack,
            config: HashMap::new(),
            enabled: true,
        };
        let channel_id = manager.add_channel(channel);

        // Create escalation policy
        let policy = EscalationPolicy::new(
            "Critical Policy".to_string(),
            vec![EscalationLevel {
                level: 1,
                channels: vec![channel_id],
                delay_secs: 0,
            }],
        );
        let policy_id = manager.add_escalation_policy(policy);

        // Create routing rule for critical alerts
        let routing = RoutingRule {
            id: Uuid::new_v4(),
            severity_filter: Some(Severity::Critical),
            label_filter: HashMap::new(),
            escalation_policy_id: policy_id,
        };
        manager.add_routing_rule(routing);

        // Fire critical alert
        let rule = AlertRule::new(
            "Critical Error".to_string(),
            "Test".to_string(),
            "errors".to_string(),
            Operator::GreaterThan,
            100.0,
            0,
            Severity::Critical,
        );
        manager.add_rule(rule);

        let fired = manager.evaluate_metric("errors", 150.0);
        let alert_id = fired[0];

        // Route alert
        let channels = manager.route_alert(alert_id);
        assert_eq!(channels.len(), 1);
        assert_eq!(channels[0], channel_id);
    }

    #[test]
    fn test_get_alerts_by_severity() {
        let manager = AlertManager::new();

        let rule1 = AlertRule::new(
            "Warning".to_string(),
            "Test".to_string(),
            "metric1".to_string(),
            Operator::GreaterThan,
            50.0,
            0,
            Severity::Warning,
        );

        let rule2 = AlertRule::new(
            "Critical".to_string(),
            "Test".to_string(),
            "metric2".to_string(),
            Operator::GreaterThan,
            90.0,
            0,
            Severity::Critical,
        );

        manager.add_rule(rule1);
        manager.add_rule(rule2);

        manager.evaluate_metric("metric1", 60.0);
        manager.evaluate_metric("metric2", 95.0);

        let warnings = manager.get_alerts_by_severity(&Severity::Warning);
        assert_eq!(warnings.len(), 1);

        let criticals = manager.get_alerts_by_severity(&Severity::Critical);
        assert_eq!(criticals.len(), 1);
    }
}
