self.addEventListener('install', ()=>self.skipWaiting());
self.addEventListener('activate', e=>e.waitUntil(self.clients.claim()));

self.addEventListener('fetch', e=>{
    const req = e.request;
    const url = new URL(req.url);
    if (url.origin != location.origin) return;

    const isNavigation = req.mode === 'navigate';
    if (isNavigation) {
        e.respondWith(
            caches.match(req).then(cached=>{
                if (cached) return cached;
                return fetch(req).then(resp=>{
                    if(resp.ok) caches.open('pet-info').then(c=>c.put(req,resp.clone())).catch(()=>{});
                    return resp;
                });
            })
        );
        return;
    }
    e.respondWith(
        caches.match(req).then(cached=>{
            if (cached) return cached;
            return fetch(req).then(resp => {
                if (resp.ok) caches.open('pet-info').then(c=>c.put(req, resp.clone())).catch(()=>{});
                return resp;                
            });
        })
    );
});
