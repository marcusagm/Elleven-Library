import Database from "@tauri-apps/plugin-sql";

let db: Database | null = null;

export async function getDb() {
  if (db) return db;
  db = await Database.load("sqlite:elleven.db");
  return db;
}

export async function initDb() {
  const database = await getDb();
  
  // We'll run the schema. Since we can't easily read the .sql file 
  // directly in the frontend without some setup, we'll embed the essential init SQL here.
  const schema = `
    CREATE TABLE IF NOT EXISTS locations (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        path TEXT NOT NULL UNIQUE,
        name TEXT NOT NULL,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP
    );

    CREATE TABLE IF NOT EXISTS images (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        location_id INTEGER NOT NULL,
        path TEXT NOT NULL UNIQUE,
        filename TEXT NOT NULL,
        width INTEGER,
        height INTEGER,
        size INTEGER,
        thumbnail_path TEXT,
        format TEXT,
        hash TEXT,
        created_at DATETIME NOT NULL,
        modified_at DATETIME NOT NULL,
        added_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (location_id) REFERENCES locations(id) ON DELETE CASCADE
    );

    CREATE TABLE IF NOT EXISTS tags (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL UNIQUE,
        parent_id INTEGER,
        color TEXT,
        order_index INTEGER DEFAULT 0,
        created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (parent_id) REFERENCES tags(id) ON DELETE SET NULL
    );

    CREATE TABLE IF NOT EXISTS image_tags (
        image_id INTEGER NOT NULL,
        tag_id INTEGER NOT NULL,
        PRIMARY KEY (image_id, tag_id),
        FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE,
        FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
    );
  `;

  // Split and execute each statement
  const statements = schema
    .split(';')
    .map(s => s.trim())
    .filter(s => s.length > 0);

  for (const statement of statements) {
    await database.execute(statement);
  }

  // Schema patches for existing DBs
  try { await database.execute("ALTER TABLE tags ADD COLUMN order_index INTEGER DEFAULT 0;"); } catch (e) {}
  try { await database.execute("ALTER TABLE images ADD COLUMN thumbnail_path TEXT;"); } catch (e) {}
  try { await database.execute("ALTER TABLE images ADD COLUMN format TEXT;"); } catch (e) {}
  try { await database.execute("ALTER TABLE images ADD COLUMN rating INTEGER DEFAULT 0;"); } catch (e) {}
  try { await database.execute("ALTER TABLE images ADD COLUMN notes TEXT;"); } catch (e) {}

  console.log("Database initialized successfully.");
  return database;
}

export async function addLocation(path: string, name: string) {
  const database = await getDb();
  return await database.execute(
    "INSERT OR IGNORE INTO locations (path, name) VALUES ($1, $2)",
    [path, name]
  );
}

export async function getLocations() {
  const database = await getDb();
  return await database.select<{ id: number; path: string; name: string }[]>(
    "SELECT * FROM locations"
  );
}

export async function getImages(limit: number = 1000, offset: number = 0) {
  const database = await getDb();
  return await database.select<any[]>(
    "SELECT id, path, filename, width, height, size, thumbnail_path, format, rating, notes, created_at, modified_at FROM images ORDER BY id ASC LIMIT $1 OFFSET $2",
    [limit, offset]
  );
}
