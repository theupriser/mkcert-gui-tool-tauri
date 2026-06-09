const { invoke } = window.__TAURI__.core;

const statusBanner = document.getElementById('status-banner');
const statusText = document.getElementById('status-text');
const btnInstallCa = document.getElementById('btn-install-ca');
const btnOpenCaFolder = document.getElementById('btn-open-ca-folder');
const btnBrowse = document.getElementById('btn-browse');
const btnGenerate = document.getElementById('btn-generate');
const domainsInput = document.getElementById('domains');
const directoryInput = document.getElementById('directory');
const autoScheduleCheckbox = document.getElementById('auto-schedule');
const logOutput = document.getElementById('log');

let selectedDirectory = '';

async function initApp() {
  try {
    const savedSettings = await invoke('load_settings');
    if (savedSettings) {
      if (savedSettings.domains) domainsInput.value = savedSettings.domains;
      if (savedSettings.directory) {
        selectedDirectory = savedSettings.directory;
        directoryInput.value = selectedDirectory;
      }
      if (savedSettings.auto_schedule !== undefined) {
        autoScheduleCheckbox.checked = savedSettings.auto_schedule;
      }
    }
    checkCAStatus();
  } catch (e) {
    logOutput.innerText = `❌ Error initializing app: ${e}`;
  }
}

async function checkCAStatus() {
  try {
    const status = await invoke('check_root_ca');
    statusBanner.style.display = 'flex';
    
    if (status.installed) {
      statusBanner.className = 'status-banner status-success';
      statusText.innerText = '✅ mkcert & Root CA are successfully installed and trusted.';
      btnOpenCaFolder.style.display = 'inline-flex';
      btnInstallCa.style.display = 'none';
      btnGenerate.disabled = false;
      btnGenerate.style.opacity = '1';
      logOutput.innerText = 'Ready for use.';
    } else if (status.missing_binary) {
      statusBanner.className = 'status-banner status-danger';
      statusText.innerText = status.error;
      btnOpenCaFolder.style.display = 'none';
      btnInstallCa.style.display = 'none';
      btnGenerate.disabled = true;
      btnGenerate.style.opacity = '0.5';
      logOutput.innerText = '⚠️ Action required: Install mkcert on your local machine.';
    } else {
      statusBanner.className = 'status-banner status-danger';
      statusText.innerText = '❌ Local Root CA is NOT installed or trusted!';
      btnOpenCaFolder.style.display = 'inline-flex';
      btnInstallCa.style.display = 'block';
      btnGenerate.disabled = false;
      btnGenerate.style.opacity = '1';
      logOutput.innerText = '⚠️ Hint: Click "Install Now" to trust certificates locally.';
    }
  } catch (e) {
    logOutput.innerText = `❌ Error checking CA status: ${e}`;
  }
}

initApp();

btnInstallCa.addEventListener('click', async () => {
  logOutput.innerText = 'Installing Local Root CA...';
  try {
    const result = await invoke('install_root_ca');
    if (result.success) {
      logOutput.innerText = `✅ Root CA successfully trusted!\n\n${result.message}`;
      checkCAStatus();
    } else { logOutput.innerText = `❌ Installation failed:\n\n${result.message}`; }
  } catch (e) { logOutput.innerText = `❌ Error: ${e}`; }
});

btnOpenCaFolder.addEventListener('click', async () => {
  logOutput.innerText = 'Opening mkcert root certificate folder...';
  try {
    const result = await invoke('open_root_ca_folder');
    if (result.success) {
      logOutput.innerText = `✅ ${result.message}`;
    } else {
      logOutput.innerText = `❌ Could not open root certificate folder:\n\n${result.message}`;
    }
  } catch (e) {
    logOutput.innerText = `❌ Error opening root certificate folder: ${e}`;
  }
});

btnBrowse.addEventListener('click', async () => {
  try {
    const dir = await invoke('select_directory');
    if (dir) {
      selectedDirectory = dir; 
      directoryInput.value = selectedDirectory; 
    }
  } catch (e) { logOutput.innerText = `❌ Error browsing folder: ${e}`; }
});

btnGenerate.addEventListener('click', async () => {
  logOutput.innerText = 'Generating local SSL certificates...';
  try {
    const result = await invoke('generate_certs', { 
      domainsString: domainsInput.value, 
      targetDir: selectedDirectory 
    });
    
    if (result.success) {
      logOutput.innerText = `✅ Success!\n\n${result.message}`;
      
      await invoke('save_settings', {
        settings: {
          domains: domainsInput.value,
          directory: selectedDirectory,
          auto_schedule: autoScheduleCheckbox.checked
        }
      });
      
      if (autoScheduleCheckbox.checked) {
        logOutput.innerText += `\n\n⏰ Automated execution scheduled daily at 00:00!`;
      } else {
        logOutput.innerText += `\n\nManual execution mode enabled.`;
      }
    } else { logOutput.innerText = `❌ Error occurred:\n\n${result.message}`; }
  } catch (e) { logOutput.innerText = `❌ Execution error: ${e}`; }
});
