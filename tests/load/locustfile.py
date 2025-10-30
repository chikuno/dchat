# Locust load testing for dchat
# Run with: locust -f locustfile.py --host=http://localhost:7071

from locust import HttpUser, task, between
import json
import time
import random
import string

class DchatUser(HttpUser):
    wait_time = between(1, 3)  # Random wait between 1-3 seconds
    
    def on_start(self):
        """Called when a simulated user starts"""
        self.user_id = f"user-{''.join(random.choices(string.ascii_lowercase, k=8))}"
        self.message_count = 0
        
    def generate_message(self):
        """Generate a random test message"""
        self.message_count += 1
        return {
            "id": f"{self.user_id}-{self.message_count}",
            "sender": self.user_id,
            "content": f"Test message {self.message_count} from {self.user_id}",
            "timestamp": int(time.time() * 1000),
        }
    
    @task(3)
    def send_message(self):
        """Send a message to relay (weight: 3)"""
        message = self.generate_message()
        with self.client.post(
            "/api/v1/messages",
            json=message,
            catch_response=True,
            name="Send Message"
        ) as response:
            if response.status_code == 200:
                response.success()
            else:
                response.failure(f"Got status {response.status_code}")
    
    @task(2)
    def get_messages(self):
        """Retrieve messages (weight: 2)"""
        with self.client.get(
            f"/api/v1/messages?user={self.user_id}",
            catch_response=True,
            name="Get Messages"
        ) as response:
            if response.status_code == 200:
                response.success()
            else:
                response.failure(f"Got status {response.status_code}")
    
    @task(1)
    def get_relay_status(self):
        """Check relay status (weight: 1)"""
        with self.client.get(
            "/api/v1/status",
            catch_response=True,
            name="Relay Status"
        ) as response:
            if response.status_code == 200:
                response.success()
            else:
                response.failure(f"Got status {response.status_code}")
    
    @task(1)
    def health_check(self):
        """Health check endpoint (weight: 1)"""
        with self.client.get(
            "/health",
            catch_response=True,
            name="Health Check"
        ) as response:
            if response.status_code == 200:
                response.success()
            else:
                response.failure(f"Got status {response.status_code}")


class HeavyUser(HttpUser):
    """Simulates a heavy user sending many messages"""
    wait_time = between(0.5, 1.5)
    weight = 1  # 1 heavy user for every 10 normal users
    
    def on_start(self):
        self.user_id = f"heavy-user-{''.join(random.choices(string.ascii_lowercase, k=8))}"
        self.message_count = 0
    
    @task
    def rapid_fire_messages(self):
        """Send multiple messages rapidly"""
        for _ in range(5):
            self.message_count += 1
            message = {
                "id": f"{self.user_id}-{self.message_count}",
                "sender": self.user_id,
                "content": f"Rapid message {self.message_count}",
                "timestamp": int(time.time() * 1000),
            }
            self.client.post("/api/v1/messages", json=message)
            time.sleep(0.1)


class BurstUser(HttpUser):
    """Simulates bursty traffic patterns"""
    wait_time = between(5, 10)  # Longer wait between bursts
    weight = 0.5  # Half weight compared to normal users
    
    def on_start(self):
        self.user_id = f"burst-user-{''.join(random.choices(string.ascii_lowercase, k=8))}"
        self.message_count = 0
    
    @task
    def message_burst(self):
        """Send a burst of messages"""
        burst_size = random.randint(10, 20)
        for _ in range(burst_size):
            self.message_count += 1
            message = {
                "id": f"{self.user_id}-{self.message_count}",
                "sender": self.user_id,
                "content": f"Burst message {self.message_count}",
                "timestamp": int(time.time() * 1000),
            }
            self.client.post("/api/v1/messages", json=message)
            time.sleep(0.05)
