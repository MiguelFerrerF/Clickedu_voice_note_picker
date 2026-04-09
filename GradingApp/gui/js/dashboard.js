lucide.createIcons();
let isListening = false, isDarkMode = false, isSoundEnabled = true;

const students = [
  { id: 1, name: "Alonso Fernández, Javier", grade: "6.5" },
  { id: 2, name: "Gómez Ruiz, Elena", grade: "" },
  { id: 3, name: "Martínez López, Carlos", grade: "" },
  { id: 4, name: "Zaragoza Vega, Lucía", grade: "9.0" },
  { id: 5, name: "Díaz Carrillo, Mario", grade: "3.5" },
  { id: 6, name: "López, Andrea", grade: "" },
  { id: 7, name: "Sánchez, Marcos", grade: "7.0" },
  { id: 8, name: "Vidal, Sofía", grade: "" }
];

const studentListEl = document.getElementById('studentList');
const dictationBar = document.getElementById('dictationBar');
const micIconBar = document.getElementById('micIconBar');
const listeningWaves = document.getElementById('listeningWaves');
const dictationText = document.getElementById('dictationText');
const toastContainer = document.getElementById('toastContainer');

function renderStudents() {
  studentListEl.innerHTML = '';
  students.forEach(student => {
    const initials = student.name.split(',')[0].substring(0, 2).toUpperCase();
    let gradeDisplay = '';
    let gradeColorClass = '';

    if (student.grade === "") {
      gradeDisplay = `<span class="text-zinc-300 dark:text-zinc-700 font-normal">—</span>`;
    } else {
      const gradeNum = parseFloat(student.grade);
      if (gradeNum >= 8.0) gradeColorClass = 'text-brand-600 dark:text-brand-400 font-black';
      else if (gradeNum < 5.0) gradeColorClass = 'text-red-500 dark:text-red-400 font-bold';
      else gradeColorClass = 'text-zinc-900 dark:text-white font-bold';
      gradeDisplay = `<span class="${gradeColorClass}">${student.grade}</span>`;
    }

    studentListEl.innerHTML += `
                <!-- Tarjeta interior vuelve a ser súper curva (rounded-2xl) -->
                <li class="student-row flex items-center justify-between px-4 py-3 rounded-2xl hover:bg-zinc-100/80 dark:hover:bg-zinc-900/80 cursor-pointer group border border-transparent dark:border-zinc-700/50 dark:hover:border-zinc-700" id="student-${student.id}">
                    <div class="flex items-center gap-4 min-w-0 flex-1 mr-4">
                        <div class="w-10 h-10 rounded-full flex-shrink-0 bg-zinc-200/50 dark:bg-zinc-800 text-zinc-500 dark:text-zinc-400 flex items-center justify-center text-xs font-bold group-hover:bg-white dark:group-hover:bg-zinc-700 transition-colors shadow-sm">
                            ${initials}
                        </div>
                        <span class="font-medium text-zinc-700 dark:text-zinc-200 group-hover:text-zinc-900 dark:group-hover:text-white transition-colors truncate" title="${student.name}">
                            ${student.name}
                        </span>
                    </div>
                    <div class="w-16 flex-shrink-0 flex justify-center text-xl tabular-nums tracking-tight" id="grade-container-${student.id}">
                        ${gradeDisplay}
                    </div>
                </li>`;
  });
}

function showToast(message, type = 'info') {
  const toast = document.createElement('div');
  toast.className = `flex items-center gap-3 px-4 py-2.5 rounded-full text-sm font-medium shadow-xl opacity-0 transform translate-x-10 transition-all duration-400 ease-ios pointer-events-auto bg-zinc-900 dark:bg-zinc-100 text-white dark:text-zinc-900`;
  const icon = type === 'success' ? "check-circle-2" : (type === 'error' ? "alert-triangle" : "info");
  const iconColor = type === 'success' ? "text-brand-400 dark:text-brand-600" : (type === 'error' ? "text-red-400 dark:text-red-600" : "text-zinc-400 dark:text-zinc-500");

  toast.innerHTML = `<i data-lucide="${icon}" class="w-4 h-4 ${iconColor}"></i><span>${message}</span>`;
  toastContainer.appendChild(toast);
  lucide.createIcons();

  requestAnimationFrame(() => toast.classList.remove('opacity-0', 'translate-x-10'));
  setTimeout(() => {
    toast.classList.add('opacity-0', 'scale-90', '-translate-y-2');
    setTimeout(() => toast.remove(), 400);
  }, 2500);
}

// --- LÓGICA DEL BOTÓN MAXIMIZAR (MORPHING) Y SIMULACIÓN NATIVA ---
let isMaximized = false;

function windowControls(action) {
  if (action === 'minimize') {
    if (window.pywebview) window.pywebview.api.minimize_window();
  }
  else if (action === 'maximize') {
    const iconMax = document.getElementById('iconMaximize');
    const iconRestore = document.getElementById('iconRestore');

    if (!isMaximized) {
      isMaximized = true;
      iconMax.classList.replace('scale-100', 'scale-50');
      iconMax.classList.replace('opacity-100', 'opacity-0');
      iconRestore.classList.replace('scale-50', 'scale-100');
      iconRestore.classList.replace('opacity-0', 'opacity-100');
      if (window.pywebview) window.pywebview.api.toggle_maximize();
    } else {
      isMaximized = false;
      iconRestore.classList.replace('scale-100', 'scale-50');
      iconRestore.classList.replace('opacity-100', 'opacity-0');
      iconMax.classList.replace('scale-50', 'scale-100');
      iconMax.classList.replace('opacity-0', 'opacity-100');
      if (window.pywebview) window.pywebview.api.toggle_maximize();
    }
  }
  else if (action === 'close') {
    if (window.pywebview) window.pywebview.api.close_window();
  }
}

// --- LÓGICA DE DICTADO ---
function startListening() {
  if (isListening || document.getElementById('cmdOverlay').classList.contains('modal-overlay-enter')) return;
  isListening = true;
  dictationBar.classList.replace('bg-zinc-700', 'bg-brand-500');
  dictationBar.classList.replace('dark:bg-zinc-300', 'dark:bg-brand-500');
  dictationBar.classList.replace('text-white', 'text-white');
  dictationBar.classList.replace('dark:text-zinc-900', 'dark:text-white');
  dictationBar.classList.add('scale-105');
  micIconBar.classList.add('opacity-0');
  listeningWaves.classList.remove('hidden');
  setTimeout(() => listeningWaves.classList.replace('scale-50', 'scale-100'), 10);
  setTimeout(() => listeningWaves.classList.replace('opacity-0', 'opacity-100'), 10);
  dictationText.innerHTML = "Escuchando nota...";
}

function stopListening() {
  if (!isListening) return; isListening = false;
  dictationBar.classList.replace('bg-brand-500', 'bg-zinc-700');
  dictationBar.classList.replace('dark:bg-brand-500', 'dark:bg-zinc-300');
  dictationBar.classList.replace('dark:text-white', 'dark:text-zinc-900');
  dictationBar.classList.remove('scale-105');
  micIconBar.classList.remove('opacity-0');
  listeningWaves.classList.replace('scale-100', 'scale-50');
  listeningWaves.classList.replace('opacity-100', 'opacity-0');
  setTimeout(() => listeningWaves.classList.add('hidden'), 300);
  dictationText.innerHTML = `Mantén <kbd class="mx-1 px-2 py-0.5 bg-zinc-600 dark:bg-zinc-200 rounded text-xs font-sans text-zinc-200 dark:text-zinc-700 font-bold uppercase transition-colors">Espacio</kbd> para evaluar`;

  const ungraded = students.filter(s => s.grade === "");
  if (ungraded.length > 0) {
    const targetStudent = ungraded[Math.floor(Math.random() * ungraded.length)];
    targetStudent.grade = (Math.random() > 0.8) ? (Math.random() * 4).toFixed(1) : (Math.random() * 3 + 6.5).toFixed(1);
    renderStudents();
    const rowEl = document.getElementById(`student-${targetStudent.id}`);
    const gradeContainer = document.getElementById(`grade-container-${targetStudent.id}`);
    rowEl.classList.add('just-updated');
    gradeContainer.firstElementChild.classList.add('animate-slide-up-fade');
    setTimeout(() => rowEl.classList.remove('just-updated'), 600);
    const firstName = targetStudent.name.split(',')[1].trim();
    showToast(`Evaluado: ${firstName}`, 'success');
  } else {
    showToast("Todos evaluados", "info");
  }
}

window.addEventListener('keydown', (e) => { if (e.code === 'Space' && !e.repeat && document.activeElement.tagName !== 'INPUT') { e.preventDefault(); startListening(); } });
window.addEventListener('keyup', (e) => { if (e.code === 'Space') { e.preventDefault(); stopListening(); } });

// Command Palette
const cmdOverlay = document.getElementById('cmdOverlay');
const cmdModal = document.getElementById('cmdModal');
const cmdInput = document.getElementById('cmdInput');
const cmdItems = document.querySelectorAll('.cmd-item');

function toggleTheme() {
  isDarkMode = !isDarkMode;
  document.documentElement.classList.toggle('dark', isDarkMode);
  document.getElementById('cmdMoonIcon').setAttribute('data-lucide', isDarkMode ? 'sun' : 'moon');
  lucide.createIcons();
  renderStudents();
}

function openModal(overlay, modal, inputToFocus = null) {
  overlay.classList.remove('hidden');
  requestAnimationFrame(() => {
    overlay.classList.add('modal-overlay-enter');
    modal.classList.add('modal-content-enter');
    if (inputToFocus) { inputToFocus.value = ''; inputToFocus.focus(); }
  });
}

function closeModal(overlay, modal) {
  overlay.classList.remove('modal-overlay-enter');
  modal.classList.remove('modal-content-enter');
  setTimeout(() => overlay.classList.add('hidden'), 300);
}

window.addEventListener('keydown', (e) => {
  if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
    e.preventDefault();
    cmdOverlay.classList.contains('hidden') ? openModal(cmdOverlay, cmdModal, cmdInput) : closeModal(cmdOverlay, cmdModal);
  }
  if (e.key === 'Escape') {
    if (!cmdOverlay.classList.contains('hidden')) closeModal(cmdOverlay, cmdModal);
  }
  // Manual Snap shortcuts: Alt + ArrowLeft / Alt + ArrowRight / ArrowUp / ArrowDown
  if (e.altKey && e.key === 'ArrowLeft') {
    e.preventDefault();
    if (window.pywebview) window.pywebview.api.snap_left();
  }
  if (e.altKey && e.key === 'ArrowRight') {
    e.preventDefault();
    if (window.pywebview) window.pywebview.api.snap_right();
  }
  if (e.altKey && e.key === 'ArrowUp') {
    e.preventDefault();
    if (window.pywebview) window.pywebview.api.snap_top();
  }
  if (e.altKey && e.key === 'ArrowDown') {
    e.preventDefault();
    if (window.pywebview) window.pywebview.api.snap_bottom();
  }
});

document.getElementById('openCmdBtn').addEventListener('click', () => openModal(cmdOverlay, cmdModal, cmdInput));
cmdOverlay.addEventListener('click', (e) => { if (e.target === cmdOverlay) closeModal(cmdOverlay, cmdModal); });

// LÓGICA DE BÚSQUEDA EN COMMAND PALETTE
const emptySearchMsg = document.createElement('div');
emptySearchMsg.className = "px-6 py-10 text-center hidden flex flex-col items-center justify-center text-zinc-400";
emptySearchMsg.innerHTML = `<i data-lucide="search-x" class="w-8 h-8 mb-3 opacity-20"></i><p class="text-sm font-medium">Buscador sin resultados</p>`;
document.getElementById('cmdList').appendChild(emptySearchMsg);

cmdInput.addEventListener('input', (e) => {
  const query = e.target.value.toLowerCase().trim();
  const items = document.querySelectorAll('.cmd-item');
  const separators = document.querySelectorAll('#cmdList > div.h-px');
  let hasResults = false;

  items.forEach(item => {
    const text = item.innerText.toLowerCase();
    const isVisible = text.includes(query);
    item.classList.toggle('hidden', !isVisible);
    if (isVisible) hasResults = true;
  });

  // Ocultar separadores si estamos buscando
  separators.forEach(sep => {
    sep.classList.toggle('hidden', query.length > 0);
  });

  emptySearchMsg.classList.toggle('hidden', hasResults || query.length === 0);
  lucide.createIcons();
});

// Re-delegar eventos para nuevos items del Command Palette
document.getElementById('cmdList').addEventListener('click', (e) => {
  const item = e.target.closest('.cmd-item');
  if (!item) return;

  const action = item.getAttribute('data-action');
  if (action === 'toggle-theme') toggleTheme();
  if (action === 'snap-left') window.pywebview.api.snap_left();
  if (action === 'snap-right') window.pywebview.api.snap_right();
  if (action === 'snap-top') window.pywebview.api.snap_top();
  if (action === 'snap-bottom') window.pywebview.api.snap_bottom();
  if (action === 'logout') logout();
  if (action === 'login') window.location.href = 'login.html';

  closeModal(cmdOverlay, cmdModal);
});

// --- SISTEMA DE PERFIL Y SESIÓN (Moved to Command Palette) ---
function initProfile() {
  const user = localStorage.getItem('currentUser');
  const cmdLogin = document.getElementById('cmdLoginBtn');
  const cmdLogout = document.getElementById('cmdLogoutBtn');

  if (!user || user === 'Invitado') {
    cmdLogin.classList.remove('hidden');
    cmdLogout.classList.add('hidden');
  } else {
    cmdLogin.classList.add('hidden');
    cmdLogout.classList.remove('hidden');
    cmdLogout.querySelector('span').textContent = `Cerrar Sesión (${user})`;
  }
  lucide.createIcons();
}

function logout() {
  showToast("Cerrando sesión...", "info");
  localStorage.removeItem('currentUser');
  setTimeout(() => {
    document.body.style.opacity = '0';
    setTimeout(() => window.location.href = 'login.html', 400);
  }, 500);
}


// renderStudents() se llama abajo
initProfile();
renderStudents();
