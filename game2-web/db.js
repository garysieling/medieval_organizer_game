// IndexedDB, the datomic save's shelf: the embedded harness keeps its
// journal replay in one slot and reloads to the same village next visit.
// The remote client persists nothing here - the server owns that world;
// its memory rides in the Hello's claim token (localStorage).

const NAME = 'game2';
const STORE = 'saves';

function open() {
  return new Promise((res, rej) => {
    const r = indexedDB.open(NAME, 1);
    r.onupgradeneeded = () => r.result.createObjectStore(STORE);
    r.onsuccess = () => res(r.result);
    r.onerror = () => rej(r.error);
  });
}

export async function load(key) {
  try {
    const d = await open();
    return await new Promise(res => {
      const q = d.transaction(STORE).objectStore(STORE).get(key);
      q.onsuccess = () => res(q.result ?? null);
      q.onerror = () => res(null);
    });
  } catch (_) {
    return null; // a browser without IndexedDB still plays, just forgets
  }
}

export function save(key, value) {
  open().then(d => {
    const t = d.transaction(STORE, 'readwrite');
    t.objectStore(STORE).put(value, key);
  }).catch(() => {});
}
