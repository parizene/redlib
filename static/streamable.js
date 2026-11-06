// @license http://www.gnu.org/licenses/agpl-3.0.html AGPL-3.0
document.addEventListener('click', function (event) {
    const button = event.target.closest('button[data-streamable-src]');
    if (!button) return;
    const source = button.dataset.streamableSrc;
    if (!/^https:\/\/streamable\.com\/o\/[a-zA-Z0-9]+$/.test(source)) return;
    const player = button.closest('.streamable-player');
    const frame = document.createElement('iframe');
    frame.src = source;
    frame.title = 'Streamable video player';
    frame.allow = 'autoplay; fullscreen; picture-in-picture';
    frame.allowFullscreen = true;
    frame.referrerPolicy = 'no-referrer';
    player.querySelector('img')?.remove();
    button.replaceWith(frame);
});
// @license-end
