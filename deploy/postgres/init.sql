-- ============================================================================
-- Hafiz PostgreSQL Initialization Script
-- ============================================================================
-- This script runs automatically when the PostgreSQL container starts
-- for the first time.
-- ============================================================================

-- Enable useful extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_stat_statements";

-- Create application user with limited privileges (optional)
-- The main 'novus' user is created by Docker environment variables

-- Grant permissions
GRANT ALL PRIVILEGES ON DATABASE hafiz TO novus;

-- Set timezone
SET timezone = 'UTC';

-- Log successful initialization
DO $$
BEGIN
    RAISE NOTICE 'Hafiz database initialized successfully';
END $$;
