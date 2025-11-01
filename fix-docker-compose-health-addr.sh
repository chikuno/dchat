#!/bin/bash
# Fix docker-compose-testnet.yml to put --health-addr before subcommand

cd /home/azureuser/chain/dchat

# Backup original
cp docker-compose-testnet.yml docker-compose-testnet.yml.backup

# Fix validator commands
sed -i 's/command: validator \(.*\) --health-addr 0.0.0.0:8080/command: --health-addr 0.0.0.0:8080 validator \1/' docker-compose-testnet.yml

# Fix relay commands  
sed -i 's/command: relay \(.*\) --health-addr 0.0.0.0:8080/command: --health-addr 0.0.0.0:8080 relay \1/' docker-compose-testnet.yml

# Fix user commands
sed -i 's/command: user \(.*\) --health-addr 0.0.0.0:8080/command: --health-addr 0.0.0.0:8080 user \1/' docker-compose-testnet.yml

echo "✅ Fixed docker-compose-testnet.yml"
echo "Backup saved as docker-compose-testnet.yml.backup"
