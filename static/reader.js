// Reader view: vim-style keyboard navigation + scroll progress bar.
(function() {
    var progressBar = document.getElementById('reader-progress-bar');
    var helpPanel = document.getElementById('reader-help');
    var backLink = document.querySelector('.reader-back');

    var STEP = Math.max(80, Math.round(window.innerHeight * 0.4));
    var PAGE = Math.max(200, Math.round(window.innerHeight * 0.85));
    var gPressed = false;

    function updateProgress() {
        var scrollTop = window.scrollY || document.documentElement.scrollTop;
        var docHeight = document.documentElement.scrollHeight - window.innerHeight;
        var pct = docHeight > 0 ? Math.min(100, (scrollTop / docHeight) * 100) : 0;
        if (progressBar) progressBar.style.width = pct + '%';
    }

    window.addEventListener('scroll', updateProgress);
    window.addEventListener('resize', updateProgress);
    updateProgress();

    document.addEventListener('keydown', function(e) {
        // Don't intercept keys when the user is in an input-like element.
        var tag = document.activeElement && document.activeElement.tagName;
        if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return;
        if (e.ctrlKey || e.metaKey || e.altKey) return;

        switch (e.key) {
            case 'j':
                e.preventDefault();
                window.scrollBy({ top: STEP, behavior: 'smooth' });
                break;
            case 'k':
                e.preventDefault();
                window.scrollBy({ top: -STEP, behavior: 'smooth' });
                break;
            case ' ':
                e.preventDefault();
                window.scrollBy({
                    top: e.shiftKey ? -PAGE : PAGE,
                    behavior: 'smooth'
                });
                break;
            case 'g':
                if (gPressed) {
                    e.preventDefault();
                    window.scrollTo({ top: 0, behavior: 'smooth' });
                    gPressed = false;
                } else {
                    gPressed = true;
                    setTimeout(function() { gPressed = false; }, 500);
                }
                return; // don't reset gPressed below
            case 'G':
                e.preventDefault();
                window.scrollTo({
                    top: document.documentElement.scrollHeight,
                    behavior: 'smooth'
                });
                break;
            case 'q':
            case 'Escape':
                if (backLink) {
                    e.preventDefault();
                    window.location.href = backLink.href;
                }
                break;
            case '?':
                e.preventDefault();
                if (helpPanel) {
                    helpPanel.style.display =
                        (helpPanel.style.display === 'none' || !helpPanel.style.display)
                            ? 'block'
                            : 'none';
                }
                break;
        }
        gPressed = false;
    });
})();
