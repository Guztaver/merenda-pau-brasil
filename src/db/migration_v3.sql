ALTER TABLE suppliers ADD COLUMN verification TEXT NOT NULL DEFAULT '';
ALTER TABLE suppliers ADD COLUMN verified INTEGER NOT NULL DEFAULT 0 CHECK(verified IN (0,1));
ALTER TABLE suppliers ADD COLUMN verified_by INTEGER REFERENCES users(id);
ALTER TABLE suppliers ADD COLUMN verified_at TEXT;
CREATE TABLE family_purchase_evidence(id INTEGER PRIMARY KEY,supplier INTEGER NOT NULL REFERENCES suppliers(id),document TEXT NOT NULL,kind TEXT NOT NULL,sha256 TEXT NOT NULL,stored_name TEXT NOT NULL,actor INTEGER NOT NULL REFERENCES users(id),stamp TEXT NOT NULL);
CREATE INDEX family_evidence_supplier ON family_purchase_evidence(supplier,stamp);
PRAGMA user_version=3;
