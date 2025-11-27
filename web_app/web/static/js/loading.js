document.addEventListener('DOMContentLoaded', () => {
    // Helper to find the submit button of a form
    const getSubmitButton = (form) => {
        // Buttons with type="submit", inputs with type="submit", or buttons with no type (default is submit)
        return form.querySelector('button[type="submit"], input[type="submit"], button:not([type])');
    };

    // --- HTMX Integration ---
    document.body.addEventListener('htmx:beforeRequest', (event) => {
        const target = event.detail.elt;
        // If the element is a form, find the submit button
        if (target.tagName === 'FORM') {
            const submitBtn = getSubmitButton(target);
            if (submitBtn) {
                submitBtn.setAttribute('aria-busy', 'true');
                submitBtn.disabled = true;
            }
        } else {
            // For buttons/links triggering the request
            target.setAttribute('aria-busy', 'true');
            if (target.tagName === 'BUTTON' || target.tagName === 'INPUT') {
                target.disabled = true;
            } else {
                target.style.pointerEvents = 'none';
            }
        }
    });

    const removeLoadingState = (event) => {
        const target = event.detail.elt;
        if (target.tagName === 'FORM') {
            const submitBtn = getSubmitButton(target);
            if (submitBtn) {
                submitBtn.removeAttribute('aria-busy');
                submitBtn.disabled = false;
            }
        } else {
            target.removeAttribute('aria-busy');
            if (target.tagName === 'BUTTON' || target.tagName === 'INPUT') {
                target.disabled = false;
            } else {
                target.style.pointerEvents = 'auto';
            }
        }
    };

    document.body.addEventListener('htmx:afterRequest', removeLoadingState);
    document.body.addEventListener('htmx:requestError', removeLoadingState);
    document.body.addEventListener('htmx:sendError', removeLoadingState);

    // --- Standard Form Integration ---
    document.addEventListener('submit', (event) => {
        // If it's an HTMX form, let HTMX handle it (htmx:beforeRequest will fire)
        if (event.target.hasAttribute('hx-post') || event.target.hasAttribute('hx-get') ||
            event.target.hasAttribute('hx-put') || event.target.hasAttribute('hx-delete') ||
            event.target.hasAttribute('hx-patch')) {
            return;
        }

        const form = event.target;
        const submitBtn = getSubmitButton(form);
        if (submitBtn && !event.defaultPrevented) {
            submitBtn.setAttribute('aria-busy', 'true');
            submitBtn.disabled = true;

            // Standard forms usually reload the page, so we don't need to remove the state.
        }
    });

    // --- Standard Link Integration ---
    document.addEventListener('click', (event) => {
        const link = event.target.closest('a');
        if (!link) return;

        // Ignore if it has HTMX attributes (handled by HTMX)
        if (link.hasAttribute('hx-get') || link.hasAttribute('hx-post')) return;

        // Special handling for downloads
        if (link.hasAttribute('data-download')) {
            event.preventDefault();
            const url = link.getAttribute('href');
            const filename = link.getAttribute('data-download');

            // Add loading state
            link.setAttribute('aria-busy', 'true');
            link.style.pointerEvents = 'none';

            fetch(url)
                .then(response => {
                    if (!response.ok) throw new Error('Download failed');
                    return response.blob();
                })
                .then(blob => {
                    const blobUrl = window.URL.createObjectURL(blob);
                    const a = document.createElement('a');
                    a.style.display = 'none';
                    a.href = blobUrl;
                    a.download = filename;
                    document.body.appendChild(a);
                    a.click();
                    window.URL.revokeObjectURL(blobUrl);
                    document.body.removeChild(a);
                })
                .catch(error => {
                    console.error('Download error:', error);
                    alert('Error downloading file');
                })
                .finally(() => {
                    // Remove loading state
                    link.removeAttribute('aria-busy');
                    link.style.pointerEvents = 'auto';
                });
            return;
        }

        // Ignore if it's a standard download (without data-download), target="_blank", or hash link
        if (link.hasAttribute('download') || link.target === '_blank' || link.getAttribute('href').startsWith('#') || link.getAttribute('href').startsWith('javascript:')) return;

        // Add loading state for standard navigation
        link.setAttribute('aria-busy', 'true');
        link.style.pointerEvents = 'none';
    });

    // --- Handle Browser Back/Forward Cache (bfcache) ---
    window.addEventListener('pageshow', (event) => {
        // If the page is being restored from the bfcache, reset loading states
        if (event.persisted) {
            document.querySelectorAll('[aria-busy="true"]').forEach(el => {
                el.removeAttribute('aria-busy');
                if (el.tagName === 'BUTTON' || el.tagName === 'INPUT') {
                    el.disabled = false;
                } else {
                    el.style.pointerEvents = 'auto';
                }
            });

            // Close any open dropdowns
            document.querySelectorAll('details.dropdown[open]').forEach(el => {
                el.removeAttribute('open');
            });
        }
    });
});
