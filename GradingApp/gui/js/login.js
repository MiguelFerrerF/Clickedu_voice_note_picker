lucide.createIcons();

// CONTROLES DE VENTANA
function windowControls(action) {
    if (!window.pywebview) return;
    if (action === 'minimize') window.pywebview.api.minimize_window();
    else if (action === 'maximize') window.pywebview.api.toggle_maximize();
    else if (action === 'close') window.pywebview.api.close_window();
}

// TOAST NOTIFICATIONS
function showToast(message, type = 'info') {
    const toast = document.createElement('div');
    toast.className = `flex items-center gap-3 px-4 py-2.5 rounded-full text-sm font-medium shadow-xl opacity-0 transform translate-x-10 transition-all duration-400 ease-ios pointer-events-auto bg-zinc-900 dark:bg-zinc-100 text-white dark:text-zinc-900 border border-zinc-800 dark:border-zinc-200`;
    const icon = type === 'error' ? "alert-circle" : (type === 'success' ? "check-circle-2" : "info");
    const iconColor = type === 'error' ? "text-red-400 dark:text-red-500" : (type === 'success' ? "text-brand-400 dark:text-brand-600" : "text-zinc-400");
    toast.innerHTML = `<i data-lucide="${icon}" class="w-4 h-4 ${iconColor}"></i><span>${message}</span>`;
    document.getElementById('toastContainer').appendChild(toast);
    lucide.createIcons();
    requestAnimationFrame(() => toast.classList.remove('opacity-0', 'translate-x-10'));
    setTimeout(() => {
        toast.classList.add('opacity-0', 'scale-90', '-translate-y-2');
        setTimeout(() => toast.remove(), 400);
    }, 3000);
}

// MOSTRAR/OCULTAR CONTRASEÑA
document.getElementById('togglePassBtn').addEventListener('click', () => {
    const input = document.getElementById('passInput');
    const icon = document.getElementById('togglePassIcon');
    if (input.type === 'password') {
        input.type = 'text'; icon.setAttribute('data-lucide', 'eye-off');
    } else {
        input.type = 'password'; icon.setAttribute('data-lucide', 'eye');
    }
    lucide.createIcons();
});

// DROPZONE COMPACTO
const dropzone = document.getElementById('dropzone');
const fileInput = document.getElementById('securityFile');
const dropContent = document.getElementById('dropzoneContent');
const fileState = document.getElementById('fileSelectedState');
const fileName = document.getElementById('fileNameDisplay');

['dragenter', 'dragover', 'dragleave', 'drop'].forEach(evt => dropzone.addEventListener(evt, e => { e.preventDefault(); e.stopPropagation(); }));
['dragenter', 'dragover'].forEach(evt => dropzone.addEventListener(evt, () => dropzone.classList.add('border-brand-500', 'bg-brand-50/50', 'dark:bg-brand-900/10')));
['dragleave', 'drop'].forEach(evt => dropzone.addEventListener(evt, () => dropzone.classList.remove('border-brand-500', 'bg-brand-50/50', 'dark:bg-brand-900/10')));

dropzone.addEventListener('drop', e => handleFiles(e.dataTransfer.files));
fileInput.addEventListener('change', function () { handleFiles(this.files); });

function handleFiles(files) {
    if (files.length > 0) {
        fileName.textContent = files[0].name;
        dropzone.classList.add('border-brand-500', 'bg-brand-50/50', 'dark:bg-brand-900/10', 'border-solid');
        dropzone.classList.remove('border-dashed');
        dropContent.classList.add('hidden');
        fileState.classList.remove('hidden');
        fileState.classList.add('flex');
    }
}

// LOGIN FLOW
document.getElementById('loginForm').addEventListener('submit', async (e) => {
    e.preventDefault();
    const user = document.getElementById('userInput').value;
    const pass = document.getElementById('passInput').value;
    const file = fileInput.files.length > 0;

    if (!user || !pass || !file) {
        showToast("Introduce tus credenciales y archivo .txt", "error");
        const card = document.getElementById('loginCard');
        card.classList.add('translate-x-2');
        setTimeout(() => card.classList.replace('translate-x-2', '-translate-x-2'), 100);
        setTimeout(() => card.classList.replace('-translate-x-2', 'translate-x-0'), 200);
        return;
    }

    const btn = document.getElementById('loginBtn');
    const btnText = document.getElementById('loginBtnText');
    const btnIcon = document.getElementById('loginBtnIcon');

    btn.classList.add('opacity-90', 'cursor-not-allowed');
    btnText.textContent = "Verificando...";
    btnIcon.setAttribute('data-lucide', 'loader-2');
    btnIcon.classList.add('animate-spin-slow');
    lucide.createIcons();

    if (window.pywebview) {
        try {
            const response = await window.pywebview.api.login(user, pass, fileInput.files[0].name);
            if (response.success) {
                localStorage.setItem('currentUser', user);
                showToast(`Bienvenido, ${user}`, "success");
                btnText.textContent = "Accediendo...";
                btnIcon.setAttribute('data-lucide', 'check');
                btnIcon.classList.remove('animate-spin-slow');
                lucide.createIcons();

                setTimeout(() => {
                    document.body.style.opacity = '0';
                    setTimeout(() => window.location.href = 'dashboard.html', 400);
                }, 500);
            } else {
                showToast("Error de acceso", "error");
                btn.classList.remove('opacity-90', 'cursor-not-allowed');
                btnText.textContent = "Iniciar Sesión";
                btnIcon.classList.remove('animate-spin-slow');
                lucide.createIcons();
            }
        } catch (err) {
            showToast("Error de conexión", "error");
            btn.classList.remove('opacity-90', 'cursor-not-allowed');
        }
    } else {
        setTimeout(() => {
            document.body.style.opacity = '0';
            setTimeout(() => window.location.href = 'dashboard.html', 400);
        }, 1000);
    }
});

document.getElementById('guestBtn').addEventListener('click', () => {
    localStorage.setItem('currentUser', 'Invitado');
    showToast("Iniciando como invitado...", "info");
    setTimeout(() => {
        document.body.style.opacity = '0';
        setTimeout(() => window.location.href = 'dashboard.html', 400);
    }, 800);
});

// ATAJOS DE TECLADO (Alt + Flechas para Snapping)
window.addEventListener('keydown', (e) => {
    if (e.altKey && window.pywebview) {
        if (e.code === 'ArrowLeft') { e.preventDefault(); window.pywebview.api.snap_left(); }
        else if (e.code === 'ArrowRight') { e.preventDefault(); window.pywebview.api.snap_right(); }
        else if (e.code === 'ArrowUp') { e.preventDefault(); window.pywebview.api.snap_top(); }
        else if (e.code === 'ArrowDown') { e.preventDefault(); window.pywebview.api.snap_bottom(); }
    }
});
