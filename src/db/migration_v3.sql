ALTER TABLE suppliers ADD COLUMN verification TEXT NOT NULL DEFAULT '';
ALTER TABLE suppliers ADD COLUMN verified INTEGER NOT NULL DEFAULT 0 CHECK(verified IN (0,1));
ALTER TABLE suppliers ADD COLUMN verified_by INTEGER REFERENCES users(id);
ALTER TABLE suppliers ADD COLUMN verified_at TEXT;
PRAGMA user_version=3;
