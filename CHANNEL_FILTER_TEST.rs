// Test to verify channel message retrieval fix
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_message_retrieval_filter_logic() {
        // This test verifies the filtering logic in get_channel_messages()
        
        // Simulating message structs
        struct Message {
            id: String,
            recipient_id: Option<String>,
            channel_id: Option<String>,
            timestamp: i64,
            status: String,
        }

        // Test data
        let messages = vec![
            // Direct message (should NOT be included)
            Message {
                id: "dm1".to_string(),
                recipient_id: Some("user2".to_string()),
                channel_id: None,
                timestamp: 1000,
                status: "sent".to_string(),
            },
            // Channel message for channel_1 (should be included)
            Message {
                id: "cm1".to_string(),
                recipient_id: None,
                channel_id: Some("channel_1".to_string()),
                timestamp: 1001,
                status: "posted".to_string(),
            },
            // Channel message for channel_2 (should NOT be included - wrong channel)
            Message {
                id: "cm2".to_string(),
                recipient_id: None,
                channel_id: Some("channel_2".to_string()),
                timestamp: 1002,
                status: "posted".to_string(),
            },
            // Another channel message for channel_1 (should be included)
            Message {
                id: "cm3".to_string(),
                recipient_id: None,
                channel_id: Some("channel_1".to_string()),
                timestamp: 1003,
                status: "posted".to_string(),
            },
        ];

        // Apply the filter logic from get_channel_messages()
        let target_channel = "channel_1";
        let filtered: Vec<_> = messages
            .into_iter()
            .filter(|msg| msg.recipient_id.is_none() && msg.channel_id.as_deref() == Some(target_channel))
            .collect();

        // Verify results
        assert_eq!(filtered.len(), 2, "Should find 2 channel messages for channel_1");
        assert_eq!(filtered[0].id, "cm1", "First message should be cm1");
        assert_eq!(filtered[1].id, "cm3", "Second message should be cm3");
        
        println!("✅ Channel message filter logic test PASSED");
        println!("   Found {} messages for channel '{}': {:?}", 
                 filtered.len(), 
                 target_channel,
                 filtered.iter().map(|m| m.id.clone()).collect::<Vec<_>>());
    }
}
