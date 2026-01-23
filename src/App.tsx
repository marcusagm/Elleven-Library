import { createSignal, Show, onMount } from "solid-js";
import { createStore, reconcile } from "solid-js/store";
import { open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { initDb, addLocation, getLocations, getImages } from "./lib/db";
import { VirtualMasonry } from "./components/VirtualMasonry";

interface ProgressPayload {
  total: number;
  processed: number;
  current_file: string;
}

interface ImageItem {
  id: number;
  path: string;
  filename: string;
  width: number | null;
  height: number | null;
  thumbnail_path: string | null;
}

function App() {
  const [rootPath, setRootPath] = createSignal<string | null>(null);
  const [loading, setLoading] = createSignal(true);
  const [store, setImages] = createStore<{ items: ImageItem[] }>({ items: [] });
  const [progress, setProgress] = createSignal<ProgressPayload | null>(null);

  const refreshImages = async () => {
    const rawImages = await getImages();
    if (rawImages.length > 0) {
       console.log("DEBUG: First image from DB:", rawImages[0]);
    }
    // No formatting needed as components handle custom protocols now
    setImages("items", reconcile(rawImages, { key: "id" }));
  };

  onMount(async () => {
    try {
      await initDb();
      const locations = await getLocations();
      if (locations.length > 0) {
        setRootPath(locations[0].path);
        console.log("Auto-starting index for:", locations[0].path);
        invoke("start_indexing", { path: locations[0].path })
          .catch(e => console.error("Auto-index failed:", e));
      }

      await refreshImages();

      await listen<ProgressPayload>("indexer:progress", (event) => {
        setProgress(event.payload);
        if (event.payload.processed % 10 === 0 || event.payload.processed === event.payload.total) {
          refreshImages();
        }
      });

      await listen<number>("indexer:complete", (event) => {
        console.log("Indexer complete. Total:", event.payload);
        setProgress(null);
        refreshImages();
      });

      interface ThumbnailEvent {
        id: number;
        path: string;
      }

      await listen<ThumbnailEvent>("thumbnail:ready", (event) => {
        // Optimistic update: Update only the specific item in the store
        // console.log("Thumbnail ready event:", event.payload);
        setImages(
          "items",
          (item) => item.id === event.payload.id,
          "thumbnail_path",
          event.payload.path
        );
      });
    } catch (err) {
      console.error("Initialization error:", err);
    } finally {
      setLoading(false);
    }
  });

  const handleSelectFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Reference Library Folder",
      });
      
      if (selected) {
        const path = typeof selected === 'string' ? selected : (selected as any).path;
        if (path) {
          const name = path.split(/[\/\\]/).pop() || path;
          await addLocation(path, name);
          setRootPath(path);
          invoke("start_indexing", { path })
            .catch(e => console.error("Invoke start_indexing failed:", e));
        }
      }
    } catch (err) {
      console.error("Failed to select folder:", err);
    }
  };

  return (
    <div class="app-container">
      <Show when={!loading()} fallback={<div class="grid-placeholder">Loading...</div>}>
        <Show 
          when={rootPath()} 
          fallback={
            <div class="welcome-screen">
              <h1>Elleven Library</h1>
              <p>Start by choosing a folder to monitor for visual references.</p>
              <button class="primary-btn" onClick={handleSelectFolder}>
                Initialize Library
              </button>
            </div>
          }
        >
          <div class="main-layout">
            <aside class="sidebar">
              <div class="logo-area">
                <span class="logo-text">11</span>
              </div>
              <nav class="sidebar-nav">
                <div class="nav-item active">Library</div>
                <div class="nav-item">Tags</div>
                <div class="nav-item">Settings</div>
              </nav>
            </aside>
            <main class="content">
              <header class="top-bar">
                <div class="path-display">
                  <span>{rootPath()}</span>
                  <Show when={progress() && progress()!.total > 1}>
                    <div class="indexer-status">
                      <span class="status-dot"></span>
                      Indexing {progress()?.processed} / {progress()?.total}
                    </div>
                  </Show>
                </div>
                <div class="search-area">
                  <input type="text" placeholder="Search references..." />
                </div>
              </header>
              <Show when={progress() && progress()!.total > 1}>
                <div class="progress-track">
                  <div 
                    class="progress-fill" 
                    style={{ width: `${(progress()!.processed / progress()!.total) * 100}%` }}
                  ></div>
                </div>
              </Show>
              <div class="grid-placeholder">
                <VirtualMasonry items={store.items} />
              </div>
            </main>
          </div>
        </Show>
      </Show>
    </div>
  );
}

export default App;
