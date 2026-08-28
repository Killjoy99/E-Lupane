sudo apt update

# PostgreSQL
sudo apt install postgresql postgresql-contrib

# Install Redis
sudo apt install redis-server


# Start PostgreSQL
sudo service postgresql start

# Start Redis
sudo service redis-server start

# Check status Of PG
sudo service postgresql status

# Check Redis status
redis-cli ping