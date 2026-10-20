ALTER TABLE receipts ADD COLUMN supplier_name TEXT NOT NULL DEFAULT '';
ALTER TABLE receipts ADD COLUMN supplier_document TEXT NOT NULL DEFAULT '';
ALTER TABLE receipts ADD COLUMN supplier_family INTEGER NOT NULL DEFAULT 0 CHECK(supplier_family IN (0,1));
ALTER TABLE receipts ADD COLUMN supplier_caf TEXT NOT NULL DEFAULT '';
ALTER TABLE items ADD COLUMN unit TEXT NOT NULL DEFAULT 'g';
ALTER TABLE items ADD COLUMN unit_grams INTEGER NOT NULL DEFAULT 1 CHECK(unit_grams>0);
DROP TRIGGER immutable_receipts_update;
UPDATE receipts SET
    supplier_name=(SELECT name FROM suppliers WHERE suppliers.id=receipts.supplier),
    supplier_document=(SELECT document FROM suppliers WHERE suppliers.id=receipts.supplier),
    supplier_family=(SELECT family FROM suppliers WHERE suppliers.id=receipts.supplier),
    supplier_caf=(SELECT caf FROM suppliers WHERE suppliers.id=receipts.supplier);
CREATE TRIGGER immutable_receipts_update BEFORE UPDATE ON receipts BEGIN SELECT RAISE(ABORT,'Registro imutável. Crie uma revisão ou estorno.'); END;
PRAGMA user_version=4;
