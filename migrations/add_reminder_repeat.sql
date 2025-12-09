-- Add repeat configuration columns to reminder table
ALTER TABLE reminder ADD COLUMN repeat_type TEXT DEFAULT NULL;
ALTER TABLE reminder ADD COLUMN repeat_interval INTEGER DEFAULT NULL;
