To apply a migration

```bash
psql -f 0002_add_start_date_to_goals.sql -h localhost -U simple_budget simple_budget
```

To export the schema.sql

```bash
pg_dump -s -f schema.sql -U simple_budget -h localhost simple_budget
```

Be sure to remove the "restrict" and "unrestrict" commands from the exported file.