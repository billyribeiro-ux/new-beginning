-- 0022_downloads_catalog_unique.sql
--
-- Adds a UNIQUE constraint on (product_id, platform, version) so the
-- catalog upsert path can use ON CONFLICT. The original 0016 migration
-- didn't include it because the catalog was assumed admin-only and
-- collision-free; PR #11's upsert path needs the constraint to make
-- re-imports idempotent (BACKEND.md §22 rule 15: forward-only migrations,
-- never edit a shipped file — hence the separate 0022).

CREATE UNIQUE INDEX IF NOT EXISTS downloads_catalog_product_platform_version_key
    ON downloads_catalog (product_id, platform, version);
