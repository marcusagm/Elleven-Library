import sqlite3
import os

db_path = "test.db"
if os.path.exists(db_path):
    os.remove(db_path)

conn = sqlite3.connect(db_path)

# Run migrations up to 20260310000000
migration_files = sorted([f for f in os.listdir("migrations") if f.endswith(".sql")])
for f in migration_files:
    print(f"Applying {f}")
    with open(f"migrations/{f}", "r") as mf:
        script = mf.read()
        try:
            conn.executescript(script)
        except Exception as e:
            print(f"Error in {f}: {e}")
            break

