document.addEventListener('DOMContentLoaded', () => {
    const terminal = document.getElementById('terminal-body');

    const lines = [
        { type: 'input', text: 't downloads' },
        { delay: 600, type: 'action', action: 'path_change_1' },
        { delay: 600, type: 'input', text: 't ..' },
        { delay: 600, type: 'action', action: 'path_change_2' },
        { delay: 1000, type: 'input', text: 'ti downloads' },
        { delay: 400, type: 'output', html: '<br><span class="t-gray">🚀 Jumped to: </span><span class="t-cyan" style="font-weight: bold;">/home/user/Downloads</span>' },
        { delay: 600, type: 'output', html: '<span class="t-yellow">📂 Contents (158 items):</span>' },
        { delay: 800, type: 'output', html: '  <span class="t-blue">📁 Images</span>' },
        { delay: 900, type: 'output', html: '  <span class="t-blue">📁 Documents</span>' },
        { delay: 1000, type: 'output', html: '  📄 proton-t-v0.1.0.tar.gz' },
        { delay: 1100, type: 'output', html: '  <span class="t-gray">... and 154 more</span><br>' },
        { delay: 2500, type: 'input', text: '', isFinal: true },
    ];

    async function wait(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }

    async function typeText(element, text, speed = 50) {
        for (let i = 0; i < text.length; i++) {
            element.textContent += text[i];
            await wait(speed);
        }
    }

    async function runTerminal() {
        terminal.innerHTML = '';
        let currentPrompt = 'user@dev:~$ ';

        for (const line of lines) {
            if (line.delay) {
                await wait(line.delay);
            }

            const lineDiv = document.createElement('div');

            if (line.type === 'input') {
                const promptSpan = document.createElement('span');
                promptSpan.className = 't-prompt';
                promptSpan.textContent = currentPrompt;
                lineDiv.appendChild(promptSpan);

                const cmdSpan = document.createElement('span');
                cmdSpan.className = 't-cmd';
                lineDiv.appendChild(cmdSpan);

                terminal.appendChild(lineDiv);

                if (line.text) {
                    await typeText(cmdSpan, line.text, 60);
                }

                if (line.isFinal) {
                    const cursor = document.createElement('span');
                    cursor.textContent = '█';
                    cursor.style.animation = 'blink 1s step-end infinite';
                    lineDiv.appendChild(cursor);
                }
            } else if (line.type === 'action' && line.action === 'clear') {
                terminal.innerHTML = '';
            } else if (line.type === 'action' && line.action === 'path_change_1') {
                currentPrompt = 'user@dev:~/downloads$ ';
            } else if (line.type === 'action' && line.action === 'path_change_2') {
                currentPrompt = 'user@dev:~$ ';
            } else {
                lineDiv.innerHTML = line.html;
                terminal.appendChild(lineDiv);
            }
        }
        setTimeout(runTerminal, 10000); // Loop animation reliably
    }

    const style = document.createElement('style');
    style.textContent = `
        @keyframes blink {
            0%, 100% { opacity: 1; }
            50% { opacity: 0; }
        }
    `;
    document.head.appendChild(style);

    // Copy to clipboard functionality
    document.querySelectorAll('.copy-btn').forEach(btn => {
        btn.addEventListener('click', () => {
            const mockup = btn.closest('.terminal-mockup');
            const code = mockup.querySelector('code').innerText;
            navigator.clipboard.writeText(code).then(() => {
                const originalIcon = btn.innerHTML;
                btn.innerHTML = '<svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>';
                btn.classList.add('copied');
                setTimeout(() => {
                    btn.innerHTML = originalIcon;
                    btn.classList.remove('copied');
                }, 2000);
            });
        });
    });

    setTimeout(runTerminal, 500);
});
