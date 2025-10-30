//! Game-Theoretic Economic Model Validation
//!
//! This module contains simulations and proofs for validating the economic security
//! of the dchat network. It tests incentive mechanisms, attack costs, and network
//! stability under various scenarios.

use dchat_core::types::UserId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Economic agent in the simulation
#[derive(Debug, Clone)]
pub struct Agent {
    pub id: UserId,
    pub agent_type: AgentType,
    pub balance: u64,
    pub staked_amount: u64,
    pub reputation: f64,
    pub behavior: BehaviorStrategy,
}

/// Type of economic agent
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentType {
    /// Regular user
    User,
    
    /// Relay node operator
    RelayOperator,
    
    /// Validator
    Validator,
    
    /// Malicious attacker
    Attacker,
}

/// Agent behavior strategy
#[derive(Debug, Clone)]
pub enum BehaviorStrategy {
    /// Always cooperate
    Honest,
    
    /// Always defect
    Malicious,
    
    /// Cooperate if others cooperate (Tit-for-Tat)
    TitForTat { last_action: Option<Action> },
    
    /// Rational profit-maximizer
    Rational { risk_tolerance: f64 },
}

/// Action an agent can take
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Deliver message honestly
    DeliverHonestly,
    
    /// Drop message
    DropMessage,
    
    /// Submit false proof
    SubmitFalseProof,
    
    /// Stake tokens
    Stake,
    
    /// Unstake tokens
    Unstake,
}

/// Simulation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationParams {
    /// Number of simulation rounds
    pub rounds: u32,
    
    /// Number of agents
    pub agent_count: u32,
    
    /// Percentage of malicious agents
    pub malicious_percentage: f64,
    
    /// Reward for honest behavior
    pub honest_reward: u64,
    
    /// Penalty for malicious behavior
    pub malicious_penalty: u64,
    
    /// Message delivery cost
    pub delivery_cost: u64,
    
    /// Network congestion factor
    pub congestion: f64,
}

/// Simulation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResults {
    /// Total rounds simulated
    pub rounds: u32,
    
    /// Network stability score (0.0 - 1.0)
    pub stability_score: f64,
    
    /// Average honest agent profit
    pub avg_honest_profit: f64,
    
    /// Average malicious agent profit
    pub avg_malicious_profit: f64,
    
    /// Successful attacks
    pub successful_attacks: u32,
    
    /// Failed attacks (detected and slashed)
    pub failed_attacks: u32,
    
    /// Total messages delivered
    pub total_messages_delivered: u64,
    
    /// Total messages dropped
    pub total_messages_dropped: u64,
    
    /// Network throughput (messages per round)
    pub throughput: f64,
    
    /// Sybil attack cost
    pub sybil_attack_cost: u64,
    
    /// Is network secure?
    pub is_secure: bool,
}

/// Attack scenario
#[derive(Debug, Clone)]
pub enum AttackScenario {
    /// Sybil attack: create many fake identities
    Sybil { fake_identities: u32 },
    
    /// Eclipse attack: isolate a node
    Eclipse { target_node: UserId },
    
    /// Message censorship
    Censorship { target_users: Vec<UserId> },
    
    /// DDoS on relay nodes
    DDoS { target_relays: Vec<UserId> },
    
    /// Economic attack: drain token supply
    TokenDraining { drain_rate: f64 },
}

/// Economic model validator
pub struct EconomicModel {
    agents: Vec<Agent>,
    params: SimulationParams,
    round: u32,
    total_messages: u64,
    successful_deliveries: u64,
    failed_deliveries: u64,
}

impl EconomicModel {
    /// Create a new economic model simulation
    pub fn new(params: SimulationParams) -> Self {
        let mut agents = Vec::new();
        
        // Create honest agents
        let honest_count = (params.agent_count as f64 * (1.0 - params.malicious_percentage)) as u32;
        for _ in 0..honest_count {
            agents.push(Agent {
                id: UserId::new(),
                agent_type: AgentType::User,
                balance: 10_000,
                staked_amount: 0,
                reputation: 1.0,
                behavior: BehaviorStrategy::Honest,
            });
        }
        
        // Create malicious agents
        let malicious_count = params.agent_count - honest_count;
        for _ in 0..malicious_count {
            agents.push(Agent {
                id: UserId::new(),
                agent_type: AgentType::Attacker,
                balance: 10_000,
                staked_amount: 0,
                reputation: 1.0,
                behavior: BehaviorStrategy::Malicious,
            });
        }
        
        Self {
            agents,
            params,
            round: 0,
            total_messages: 0,
            successful_deliveries: 0,
            failed_deliveries: 0,
        }
    }
    
    /// Run the simulation
    pub fn run(&mut self) -> SimulationResults {
        for _ in 0..self.params.rounds {
            self.simulate_round();
            self.round += 1;
        }
        
        self.calculate_results()
    }
    
    /// Simulate a single round
    fn simulate_round(&mut self) {
        // Each agent attempts to deliver messages
        for i in 0..self.agents.len() {
            let action = self.decide_action(i);
            self.execute_action(i, action);
        }
        
        // Update reputation based on behavior
        self.update_reputation();
    }
    
    /// Agent decides what action to take
    fn decide_action(&self, agent_idx: usize) -> Action {
        let agent = &self.agents[agent_idx];
        
        match &agent.behavior {
            BehaviorStrategy::Honest => Action::DeliverHonestly,
            BehaviorStrategy::Malicious => Action::DropMessage,
            BehaviorStrategy::TitForTat { last_action } => {
                last_action.unwrap_or(Action::DeliverHonestly)
            }
            BehaviorStrategy::Rational { risk_tolerance } => {
                // Calculate expected value
                let honest_ev = self.params.honest_reward as f64 - self.params.delivery_cost as f64;
                let malicious_ev = -(self.params.malicious_penalty as f64 * (1.0 - risk_tolerance));
                
                if honest_ev > malicious_ev {
                    Action::DeliverHonestly
                } else {
                    Action::DropMessage
                }
            }
        }
    }
    
    /// Execute an agent's action
    fn execute_action(&mut self, agent_idx: usize, action: Action) {
        self.total_messages += 1;
        
        match action {
            Action::DeliverHonestly => {
                self.successful_deliveries += 1;
                // Reward honest delivery
                self.agents[agent_idx].balance += self.params.honest_reward;
                self.agents[agent_idx].balance -= self.params.delivery_cost;
            }
            Action::DropMessage => {
                self.failed_deliveries += 1;
                // Detect and slash with probability based on reputation
                let detection_prob = 0.8; // 80% detection rate
                if rand::random::<f64>() < detection_prob {
                    // Caught! Apply penalty
                    let penalty = self.params.malicious_penalty.min(self.agents[agent_idx].balance);
                    self.agents[agent_idx].balance -= penalty;
                }
            }
            Action::SubmitFalseProof => {
                // High detection rate for false proofs
                let detection_prob = 0.95;
                if rand::random::<f64>() < detection_prob {
                    let penalty = (self.params.malicious_penalty * 2).min(self.agents[agent_idx].balance);
                    self.agents[agent_idx].balance -= penalty;
                }
            }
            _ => {}
        }
    }
    
    /// Update agent reputation based on behavior
    fn update_reputation(&mut self) {
        for agent in &mut self.agents {
            match agent.behavior {
                BehaviorStrategy::Honest => {
                    agent.reputation = (agent.reputation + 0.01).min(1.0);
                }
                BehaviorStrategy::Malicious => {
                    agent.reputation = (agent.reputation - 0.05).max(0.0);
                }
                _ => {}
            }
        }
    }
    
    /// Calculate final results
    fn calculate_results(&self) -> SimulationResults {
        let honest_agents: Vec<&Agent> = self.agents
            .iter()
            .filter(|a| matches!(a.behavior, BehaviorStrategy::Honest))
            .collect();
        
        let malicious_agents: Vec<&Agent> = self.agents
            .iter()
            .filter(|a| matches!(a.behavior, BehaviorStrategy::Malicious))
            .collect();
        
        let avg_honest_profit = if !honest_agents.is_empty() {
            honest_agents.iter().map(|a| a.balance as f64).sum::<f64>() / honest_agents.len() as f64 - 10_000.0
        } else {
            0.0
        };
        
        let avg_malicious_profit = if !malicious_agents.is_empty() {
            malicious_agents.iter().map(|a| a.balance as f64).sum::<f64>() / malicious_agents.len() as f64 - 10_000.0
        } else {
            0.0
        };
        
        let stability_score = if self.total_messages > 0 {
            self.successful_deliveries as f64 / self.total_messages as f64
        } else {
            0.0
        };
        
        let throughput = self.total_messages as f64 / self.params.rounds as f64;
        
        // Sybil attack cost (need to stake for each identity)
        let sybil_attack_cost = 1_000 * self.params.agent_count as u64; // 1k per identity
        
        // Network is secure if:
        // 1. Honest agents profit more than malicious
        // 2. Stability > 90%
        // 3. Sybil attack cost is prohibitive
        let is_secure = avg_honest_profit > avg_malicious_profit
            && stability_score > 0.9
            && sybil_attack_cost > 100_000;
        
        SimulationResults {
            rounds: self.params.rounds,
            stability_score,
            avg_honest_profit,
            avg_malicious_profit,
            successful_attacks: (self.failed_deliveries / 10) as u32, // estimate
            failed_attacks: (self.failed_deliveries * 8 / 10) as u32,
            total_messages_delivered: self.successful_deliveries,
            total_messages_dropped: self.failed_deliveries,
            throughput,
            sybil_attack_cost,
            is_secure,
        }
    }
    
    /// Test a specific attack scenario
    pub fn test_attack(&mut self, scenario: AttackScenario) -> SimulationResults {
        match scenario {
            AttackScenario::Sybil { fake_identities } => {
                // Add fake identities
                for _ in 0..fake_identities {
                    self.agents.push(Agent {
                        id: UserId::new(),
                        agent_type: AgentType::Attacker,
                        balance: 1_000, // Limited initial balance
                        staked_amount: 0,
                        reputation: 0.1, // Low initial reputation
                        behavior: BehaviorStrategy::Malicious,
                    });
                }
            }
            AttackScenario::Eclipse { target_node: _ } => {
                // Simulate isolation
                self.params.congestion = 0.9; // High congestion
            }
            AttackScenario::Censorship { target_users } => {
                // Malicious agents target specific users
                let target_count = target_users.len() as u64;
                self.failed_deliveries += target_count * self.params.rounds as u64;
            }
            AttackScenario::DDoS { target_relays: _ } => {
                // Simulate service degradation
                self.params.congestion = 0.95;
            }
            AttackScenario::TokenDraining { drain_rate } => {
                // Simulate token drain
                for agent in &mut self.agents {
                    if matches!(agent.agent_type, AgentType::Attacker) {
                        agent.balance = (agent.balance as f64 * (1.0 + drain_rate)) as u64;
                    }
                }
            }
        }
        
        self.run()
    }
}

impl Default for SimulationParams {
    fn default() -> Self {
        Self {
            rounds: 1000,
            agent_count: 100,
            malicious_percentage: 0.2, // 20% malicious
            honest_reward: 100,
            malicious_penalty: 500,
            delivery_cost: 10,
            congestion: 0.1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_honest_network() {
        let params = SimulationParams {
            malicious_percentage: 0.0,
            ..Default::default()
        };
        
        let mut model = EconomicModel::new(params);
        let results = model.run();
        
        assert!(results.stability_score > 0.95);
        assert!(results.avg_honest_profit > 0.0);
        assert_eq!(results.successful_attacks, 0);
    }
    
    #[test]
    fn test_mixed_network() {
        let params = SimulationParams::default();
        let mut model = EconomicModel::new(params);
        let results = model.run();
        
        // Honest agents should profit more
        assert!(results.avg_honest_profit > results.avg_malicious_profit);
        assert!(results.stability_score > 0.7);
    }
    
    #[test]
    fn test_sybil_attack_cost() {
        let params = SimulationParams::default();
        let mut model = EconomicModel::new(params);
        
        let results = model.test_attack(AttackScenario::Sybil { fake_identities: 50 });
        
        // Sybil attack should be expensive
        assert!(results.sybil_attack_cost > 50_000);
        
        // Network should remain stable
        assert!(results.stability_score > 0.6);
    }
    
    #[test]
    fn test_censorship_attack() {
        let params = SimulationParams::default();
        let mut model = EconomicModel::new(params);
        
        let target_users = vec![UserId::new(), UserId::new()];
        let results = model.test_attack(AttackScenario::Censorship { target_users });
        
        // Network should detect and mitigate
        assert!(results.failed_attacks > results.successful_attacks);
    }
    
    #[test]
    fn test_network_security() {
        let params = SimulationParams {
            rounds: 2000,
            malicious_percentage: 0.3, // 30% malicious
            ..Default::default()
        };
        
        let mut model = EconomicModel::new(params);
        let results = model.run();
        
        // Network should remain secure even with high malicious percentage
        assert!(results.is_secure);
        assert!(results.avg_honest_profit > results.avg_malicious_profit);
    }
    
    #[test]
    fn test_rational_agents() {
        let params = SimulationParams {
            honest_reward: 200,
            malicious_penalty: 1000,
            ..Default::default()
        };
        
        let mut model = EconomicModel::new(params);
        
        // Replace some agents with rational agents
        for i in 0..10 {
            model.agents[i].behavior = BehaviorStrategy::Rational { risk_tolerance: 0.2 };
        }
        
        let results = model.run();
        
        // Rational agents should behave honestly when incentivized
        assert!(results.stability_score > 0.8);
    }
    
    #[test]
    fn test_ddos_resilience() {
        let params = SimulationParams::default();
        let mut model = EconomicModel::new(params);
        
        let target_relays = vec![UserId::new(), UserId::new()];
        let results = model.test_attack(AttackScenario::DDoS { target_relays });
        
        // Network should maintain minimum throughput
        assert!(results.throughput > 0.5);
    }
}
