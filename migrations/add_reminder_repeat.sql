-- Add repeat configuration columns to reminder table
ALTER TABLE reminder ADD COLUMN repeat_type TEXT DEFAULT NULL;
ALTER TABLE reminder ADD COLUMN repeat_interval INTEGER DEFAULT NULL;

-- Remove execution_id column - no longer needed since we track executions by Step Function name prefix
ALTER TABLE reminder DROP COLUMN execution_id;
