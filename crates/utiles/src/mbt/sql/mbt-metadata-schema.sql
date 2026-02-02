-- metadata table
CREATE TABLE metadata(name TEXT NOT NULL, value TEXT);
-- unique index on name
CREATE UNIQUE INDEX metadata_index ON metadata (name);
