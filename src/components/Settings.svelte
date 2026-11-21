<!--
  Settings.svelte — Settings section

  Cards: RAGE:MP path · updater.exe patches · domain-whitelist bypass ·
         RUI interface editor · about.

  The direct-connect capabilities (patches, whitelist bypass, RUI editor) are
  ported from the open-source tiny-ragemp-launcher project.
-->
<script>
  import { t, tf } from "../i18n";

  let {
    rageDir = "",
    clientReady = false,
    updateInfo = {},
    updateAvailable = false,
    updateChecked = false,
    updateChecking = false,
    serverOnline = false,
    serverPlayers = 0,
    serverMaxPlayers = 0,
    serverLoading = false,
    serverAddress = "",
    // Direct-connect diagnostics (ported from tiny-ragemp-launcher)
    patchManifest = null,
    patchStatus = null,
    whitelistStatus = null,
    connectMode = "raw",
    hostsDomain = "",
    isApplyingWhitelist = false,
    whitelistError = "",
    // Launch path
    launchTarget = "updater",
    launchDiagnostic = null,
    launchTargetError = "",
    // RUI editor
    ruiStatus = null,
    ruiConfig = null,
    ruiBusy = false,
    ruiError = "",
    ruiPreview = null,
    ruiLastReport = null,
    selectRageDir = async () => {},
    autoFindRageDir = async () => {},
    openUpdateModal = () => {},
    checkUpdates = async () => {},
    refreshServerStatus = async () => {},
    loadPatchStatus = async () => {},
    loadWhitelistStatus = async () => {},
    setConnectMode = async () => {},
    applyWhitelistBypass = async () => {},
    removeWhitelistBypass = async () => {},
    setLaunchTarget = async () => {},
    setRuiAutoConnect = async () => {},
    loadLaunchDiagnostic = async () => {},
    clearLaunchRegistry = async () => {},
    setServerAddress = async () => "",
    loadRuiStatus = async () => {},
    loadRuiConfig = async () => {},
    previewRuiConfig = async () => {},
    applyRuiConfig = async () => {},
    restoreRui = async () => {},
    setRuiField = () => {},
    setAutoConnectField = () => {},
    playHoverSound = () => {},
  } = $props();

  // Pull fresh diagnostics when the section mounts.
  //
  // Sequential, not parallel: each of these touches the filesystem, and two of
  // them read the 15 MB index.rui. Firing all five at once caused a burst of
  // work that made the window appear to hang.
  $effect(() => {
    let cancelled = false;
    (async () => {
      await loadLaunchDiagnostic();
      if (cancelled) return;
      await loadWhitelistStatus();
      if (cancelled) return;
      await loadPatchStatus();
      if (cancelled) return;
      await loadRuiStatus();
      if (cancelled) return;
      await loadRuiConfig();
    })();
    return () => { cancelled = true; };
  });

  const inputCls =
    "w-full bg-black/40 rounded-lg px-3 py-2 text-xs text-white/70 border border-white/5 outline-none focus:border-[#f64a46]/40 transition-colors";
  const btnGhost =
    "px-3 py-1 bg-white/5 hover:bg-white/10 rounded text-xs text-white/40 hover:text-white/60 transition-colors border border-white/5 btn-ripple disabled:opacity-40 disabled:cursor-not-allowed";
  const btnPrimary =
    "px-4 py-2 bg-[#f64a46] hover:bg-[#ff5a56] rounded-lg text-xs font-medium transition-colors btn-ripple btn-bounce glow-hover disabled:opacity-50 disabled:cursor-not-allowed";

  function toInt(v, fallback) {
    const n = parseInt(v, 10);
    return Number.isFinite(n) ? n : fallback;
  }

  // ── Server address editing ──
  let serverAddressInput = $state("");
  let serverAddressError = $state("");

  // Keep the input in sync when the store loads/changes the value.
  $effect(() => {
    serverAddressInput = serverAddress;
  });

  async function serverAddressSave() {
    serverAddressError = "";
    const msg = await setServerAddress(serverAddressInput);
    if (msg) serverAddressError = msg;
  }
</script>

<div class="w-full max-w-lg px-8 py-8 z-10 max-h-full overflow-y-auto settings-scroll">
  <h2 class="text-2xl mb-6 text-reveal" style="font-family: 'Proxima Nova Bold', sans-serif; font-weight: 700; letter-spacing: -0.48px;">
    {t("settings.title")}
  </h2>

  <!-- RAGE:MP path -->
  <div class="settings-card bg-white/5 rounded-lg p-5 mb-4 stagger-1">
    <h3 class="text-sm font-semibold text-white/70 mb-3" style="font-family: 'Proxima Nova Semibold', sans-serif;">
      {t("settings.ragePath")}
    </h3>
    <p class="text-xs text-white/30 mb-3">{t("settings.ragePathHint")}</p>
    <div class="flex items-center gap-2 mb-2">
      <div class="flex-1 bg-black/40 rounded-lg px-3 py-2 text-xs text-white/50 truncate border border-white/5">
        {rageDir || t("misc.notFound")}
      </div>
      <button class={btnPrimary} style="font-family: 'Proxima Nova Semibold', sans-serif;"
              onclick={selectRageDir} onmouseenter={playHoverSound}>
        {t("settings.select")}
      </button>
    </div>
    <div class="flex items-center gap-2 mt-2">
      <button class={btnGhost} style="font-family: 'Proxima Nova Semibold', sans-serif;"
              onclick={autoFindRageDir} onmouseenter={playHoverSound}>
        {t("settings.autoFind")}
      </button>
    </div>
    {#if clientReady}
      <p class="text-[10px] text-[#15ff00]/70">{t("settings.rageFound")}</p>
    {:else}
      <p class="text-[10px] text-[#f64a46]/70">{t("settings.rageNotFound")}</p>
    {/if}
  </div>

  <!-- Server address -->
  <div class="settings-card bg-white/5 rounded-lg p-5 mb-4 stagger-1">
    <h3 class="text-sm font-semibold text-white/70 mb-3" style="font-family: 'Proxima Nova Semibold', sans-serif;">
      {t("serverAddr.title")}
    </h3>
    <p class="text-[10px] text-white/30 mb-3">{t("serverAddr.hint")}</p>

    <div class="flex items-center gap-2">
      <input
        class={inputCls}
        type="text"
        placeholder="89.35.130.191:22005"
        value={serverAddress}
        oninput={(e) => serverAddressInput = e.currentTarget.value}
        onkeydown={(e) => { if (e.key === "Enter") serverAddressSave(); }}
        onmouseenter={playHoverSound}
      />
      <button
        class={btnPrimary}
        style="font-family: 'Proxima Nova Semibold', sans-serif;"
        onclick={serverAddressSave}
        onmouseenter={playHoverSound}>
        {t("serverAddr.save")}
      </button>
    </div>

    {#if serverAddressError}
      <p class="text-[10px] text-[#f64a46]/80 mt-2">{serverAddressError}</p>
    {/if}
  </div>

  <!-- Launch path -->
  <div class="settings-card bg-white/5 rounded-lg p-5 mb-4 stagger-2">
    <h3 class="text-sm font-semibold text-white/70 mb-3" style="font-family: 'Proxima Nova Semibold', sans-serif;">
      {t("launchPath.title")}
    </h3>
    <p class="text-[10px] text-white/30 mb-3">{t("launchPath.hint")}</p>

    <div class="flex gap-2 mb-3">
      <button
        class="flex-1 px-3 py-2 rounded-lg text-[11px] font-medium transition-colors border btn-ripple
               {launchTarget === 'updater'
                 ? 'bg-[#f64a46]/20 border-[#f64a46]/40 text-[#ff8c8c]'
                 : 'bg-white/5 border-white/5 text-white/40 hover:text-white/70'}"
        style="font-family: 'Proxima Nova Semibold', sans-serif;"
        onclick={() => setLaunchTarget("updater")} onmouseenter={playHoverSound}>
        {t("launchPath.updater")}
      </button>
      <button
        class="flex-1 px-3 py-2 rounded-lg text-[11px] font-medium transition-colors border btn-ripple
               {launchTarget === 'ragemp_v'
                 ? 'bg-[#f64a46]/20 border-[#f64a46]/40 text-[#ff8c8c]'
                 : 'bg-white/5 border-white/5 text-white/40 hover:text-white/70'}"
        style="font-family: 'Proxima Nova Semibold', sans-serif;"
        onclick={() => setLaunchTarget("ragemp_v")} onmouseenter={playHoverSound}>
        {t("launchPath.client")}
      </button>
    </div>

    <p class="text-[10px] text-white/40 mb-1">
      {launchTarget === "updater" ? t("launchPath.updaterHint") : t("launchPath.clientHint")}
    </p>
    {#if launchTarget === "ragemp_v"}
      <p class="text-[10px] text-yellow-400/70 mb-1">{t("launchPath.clientWarning")}</p>
    {/if}

    {#if launchTargetError}
      <p class="text-[10px] text-[#f64a46]/80 mb-1">{launchTargetError}</p>
    {/if}

    <!-- Diagnostics -->
    {#if launchDiagnostic}
      <div class="mt-3 pt-3 border-t border-white/5">
        <p class="text-[10px] text-white/30 mb-2">{t("launchPath.diag")}</p>

        <div class="space-y-1">
          <p class="text-[10px] text-white/45">
            {t("launchPath.diagTarget")}:
            <span class={launchDiagnostic.target_binary_present ? "text-[#15ff00]/70" : "text-[#f64a46]/80"}>
              {launchDiagnostic.target === "ragemp_v" ? "ragemp_v.exe" : "updater.exe"}
              {launchDiagnostic.target_binary_present ? t("launchPath.diagPresent") : t("launchPath.diagMissing")}
            </span>
          </p>

          <p class="text-[10px] text-white/45">
            {t("launchPath.diagRegistry")}:
            <span class={launchDiagnostic.registry_ok ? "text-[#15ff00]/70" : "text-yellow-400/70"}>
              {launchDiagnostic.registry_ok
                ? t("launchPath.diagRegistryOk")
                : t("launchPath.diagRegistryPartial")}
            </span>
            <span class={launchDiagnostic.registry_ok || launchDiagnostic.rui_pending ? "text-white/30" : "text-yellow-400/60"}>
              · {launchDiagnostic.registry_matches_server
                ? t("launchPath.diagMatches")
                : launchDiagnostic.rui_pending
                  ? t("launchPath.diagPending")
                  : t("launchPath.diagMismatch")}
            </span>
          </p>

          <p class="text-[10px] text-white/45">
            {t("launchPath.diagRui")}:
            <span class={launchDiagnostic.rui_injected ? "text-[#15ff00]/70" : launchDiagnostic.rui_pending ? "text-white/40" : "text-yellow-400/70"}>
              {launchDiagnostic.rui_injected
                ? t("launchPath.diagRuiIn")
                : launchDiagnostic.rui_pending
                  ? t("launchPath.diagRuiPending")
                  : t("launchPath.diagRuiOut")}
            </span>
          </p>

          <!-- RUI auto-connect toggle. Off by default after reports that
               injecting the auto-connect block makes the CEF UI 404. -->
          <label class="flex items-center gap-2 text-[10px] text-white/45 cursor-pointer select-none">
            <input
              type="checkbox"
              class="accent-[#f64a46] w-3 h-3"
              checked={launchDiagnostic.rui_auto_connect ?? false}
              onchange={(e) => setRuiAutoConnect(e.currentTarget.checked)}
            />
            <span>{t("launchPath.ruiToggle")}</span>
          </label>
          {#if !(launchDiagnostic.rui_auto_connect ?? false)}
            <p class="text-[10px] text-yellow-400/60 ml-5">{t("launchPath.ruiToggleHint")}</p>
          {/if}
        </div>

        <p class="text-[10px] text-white/30 mt-2 mb-1">{t("launchPath.notes")}</p>
        {#if launchDiagnostic.notes.length}
          <ul class="text-[10px] text-yellow-400/70 space-y-0.5">
            {#each launchDiagnostic.notes as n}
              <li>· {n}</li>
            {/each}
          </ul>
        {:else}
          <p class="text-[10px] text-[#15ff00]/60">{t("launchPath.noNotes")}</p>
        {/if}

        <div class="flex items-center gap-2 mt-3">
          <button class={btnGhost} onclick={loadLaunchDiagnostic} onmouseenter={playHoverSound}>
            {t("launchPath.refresh")}
          </button>
          <button class={btnGhost} onclick={clearLaunchRegistry} onmouseenter={playHoverSound}>
            {t("launchPath.clearRegistry")}
          </button>
        </div>
      </div>
    {/if}
  </div>

  <!-- updater.exe patches -->
  <div class="settings-card bg-white/5 rounded-lg p-5 mb-4 stagger-2">
    <h3 class="text-sm font-semibold text-white/70 mb-3" style="font-family: 'Proxima Nova Semibold', sans-serif;">
      {t("patches.title")}
    </h3>

    {#if patchStatus}
      <div class="flex items-center gap-2 mb-2">
        <span class="w-2 h-2 rounded-full {patchStatus.patched ? 'bg-[#15ff00]' : 'bg-yellow-400'}"></span>
        <span class="text-xs text-white/70">
          {tf("patches.applied", String(patchStatus.applied), String(patchStatus.total))}
        </span>
      </div>
      <p class="text-[10px] text-white/30 mb-2">{t("patches.applyOnPlay")}</p>
      <p class="text-[10px] text-white/30 mb-1">
        {t("patches.source")}
        {patchManifest?.source === "server" ? t("patches.sourceServer") : t("patches.sourceBuiltin")}
        {#if patchManifest?.version} · v{patchManifest.version}{/if}
      </p>
      {#if !patchStatus.reference_build && !patchStatus.patched}
        <p class="text-[10px] text-yellow-400/70 mb-1">{t("patches.referenceWarn")}</p>
      {/if}
      <p class="text-[10px] text-white/20 mb-3">
        {tf("patches.upstream", patchManifest?.upstream ?? "tiny-ragemp-launcher")}
      </p>
    {:else}
      <p class="text-[10px] text-white/30 mb-3">{t("patches.updaterMissing")}</p>
    {/if}

    <button class={btnGhost} onclick={loadPatchStatus} onmouseenter={playHoverSound}>
      {t("patches.refresh")}
    </button>
  </div>

  <!-- Domain whitelist bypass -->
  <div class="settings-card bg-white/5 rounded-lg p-5 mb-4 stagger-3">
    <h3 class="text-sm font-semibold text-white/70 mb-3" style="font-family: 'Proxima Nova Semibold', sans-serif;">
      {t("whitelist.title")}
    </h3>
    <p class="text-[10px] text-white/30 mb-3">{t("whitelist.hint")}</p>

    <div class="flex gap-2 mb-3">
      <button
        class="flex-1 px-3 py-2 rounded-lg text-[11px] font-medium transition-colors border btn-ripple
               {connectMode === 'raw'
                 ? 'bg-[#f64a46]/20 border-[#f64a46]/40 text-[#ff8c8c]'
                 : 'bg-white/5 border-white/5 text-white/40 hover:text-white/70'}"
        style="font-family: 'Proxima Nova Semibold', sans-serif;"
        onclick={() => setConnectMode("raw")} onmouseenter={playHoverSound}>
        {t("whitelist.modeRaw")}
      </button>
      <button
        class="flex-1 px-3 py-2 rounded-lg text-[11px] font-medium transition-colors border btn-ripple
               {connectMode === 'hosts'
                 ? 'bg-[#f64a46]/20 border-[#f64a46]/40 text-[#ff8c8c]'
                 : 'bg-white/5 border-white/5 text-white/40 hover:text-white/70'}"
        style="font-family: 'Proxima Nova Semibold', sans-serif;"
        onclick={() => setConnectMode("hosts")} onmouseenter={playHoverSound}>
        {t("whitelist.modeHosts")}
      </button>
    </div>

    {#if connectMode === "raw"}
      <p class="text-[10px] text-white/40 mb-1">{t("whitelist.rawHint")}</p>
      <p class="text-[10px] text-white/25 mb-3">{t("whitelist.rawHint2")}</p>
    {:else}
      <p class="text-[10px] text-white/40 mb-2">{t("whitelist.hostsHint")}</p>
      <div class="bg-black/40 rounded-lg px-3 py-2 text-[10px] text-white/50 truncate border border-white/5 mb-2">
        {hostsDomain || "astra-<ip>.gta5rp.com"}
      </div>
      {#if whitelistStatus?.hosts_mapped}
        <p class="text-[10px] text-[#15ff00]/70 mb-2">
          {tf("whitelist.mappingFound", whitelistStatus.hosts_domain, whitelistStatus.hosts_mapped_ip)}
        </p>
      {:else}
        <p class="text-[10px] text-yellow-400/70 mb-2">{t("whitelist.mappingMissing")}</p>
      {/if}
    {/if}

    {#if whitelistError}
      <p class="text-[10px] text-[#f64a46]/80 mb-2">{whitelistError}</p>
    {/if}

    <div class="flex flex-wrap items-center gap-2">
      {#if connectMode === "hosts" && !whitelistStatus?.hosts_mapped}
        <button class={btnPrimary} style="font-family: 'Proxima Nova Semibold', sans-serif;"
                onclick={applyWhitelistBypass} onmouseenter={playHoverSound} disabled={isApplyingWhitelist}>
          {isApplyingWhitelist ? t("whitelist.applying") : t("whitelist.apply")}
        </button>
      {:else if whitelistStatus?.hosts_mapped}
        <button class={btnGhost} onclick={removeWhitelistBypass} onmouseenter={playHoverSound}
                disabled={isApplyingWhitelist}>
          {isApplyingWhitelist ? t("whitelist.removing") : t("whitelist.remove")}
        </button>
      {/if}
      <button class={btnGhost} onclick={loadWhitelistStatus} onmouseenter={playHoverSound}>
        {t("whitelist.refresh")}
      </button>
    </div>

    {#if whitelistStatus?.registry_ip}
      <p class="text-[10px] text-white/20 mt-3">
        {t("whitelist.registry")} {whitelistStatus.registry_ip}:{whitelistStatus.registry_port}
      </p>
    {/if}
  </div>

  <!-- RUI interface editor -->
  <div class="settings-card bg-white/5 rounded-lg p-5 mb-4 stagger-4">
    <h3 class="text-sm font-semibold text-white/70 mb-3" style="font-family: 'Proxima Nova Semibold', sans-serif;">
      {t("rui.title")}
    </h3>
    <p class="text-[10px] text-white/30 mb-3">{t("rui.hint")}</p>

    {#if ruiStatus?.available}
      <div class="flex flex-wrap items-center gap-x-3 gap-y-1 mb-3">
        <span class="text-[10px] text-white/50">{tf("rui.state", String(ruiStatus.entries))}</span>
        <span class="text-[10px] {ruiStatus.injected ? 'text-[#15ff00]/70' : 'text-white/25'}">
          {ruiStatus.injected ? t("rui.injected") : t("rui.notInjected")}
        </span>
        <span class="text-[10px] {ruiStatus.backup_exists ? 'text-white/50' : 'text-white/25'}">
          {ruiStatus.backup_exists ? t("rui.backupExists") : t("rui.noBackup")}
        </span>
      </div>

      {#if !ruiStatus.parsable}
        <p class="text-[10px] text-yellow-400/70 mb-3">{t("rui.unparsable")}</p>
      {/if}

      {#if ruiConfig}
        <!-- Window title -->
        <label class="block text-[10px] text-white/40 mb-1" for="rui-title">{t("rui.windowTitle")}</label>
        <input id="rui-title" class={inputCls} type="text"
               placeholder={t("rui.windowTitlePlaceholder")}
               value={ruiConfig.title ?? ""}
               oninput={(e) => setRuiField({ title: e.currentTarget.value || null })}
               onmouseenter={playHoverSound} />

        <!-- Auto-connect -->
        <label class="flex items-center gap-2 mt-4 mb-2 cursor-pointer">
          <input type="checkbox" checked={ruiConfig.auto_connect.enabled}
                 onchange={(e) => setAutoConnectField({ enabled: e.currentTarget.checked })}
                 class="accent-[#f64a46]" />
          <span class="text-[11px] text-white/60">{t("rui.autoConnect")}</span>
        </label>

        {#if ruiConfig.auto_connect.enabled}
          <div class="grid grid-cols-3 gap-2 mb-2">
            <div class="col-span-2">
              <label class="block text-[10px] text-white/40 mb-1" for="rui-ip">{t("rui.autoConnectIp")}</label>
              <input id="rui-ip" class={inputCls} type="text" placeholder="89.35.130.191"
                     value={ruiConfig.auto_connect.ip}
                     oninput={(e) => setAutoConnectField({ ip: e.currentTarget.value })} />
            </div>
            <div>
              <label class="block text-[10px] text-white/40 mb-1" for="rui-port">{t("rui.autoConnectPort")}</label>
              <input id="rui-port" class={inputCls} type="number" min="1" max="65535"
                     value={ruiConfig.auto_connect.port}
                     oninput={(e) => setAutoConnectField({ port: toInt(e.currentTarget.value, 22005) })} />
            </div>
          </div>
          <div class="mb-2">
            <label class="block text-[10px] text-white/40 mb-1" for="rui-delay">{t("rui.autoConnectDelay")}</label>
            <input id="rui-delay" class={inputCls} type="number" min="0" max="60000" step="100"
                   value={ruiConfig.auto_connect.delay_ms}
                   oninput={(e) => setAutoConnectField({ delay_ms: toInt(e.currentTarget.value, 2000) })} />
          </div>
        {/if}

        <!-- Server list override -->
        <label class="flex items-center gap-2 mt-4 mb-2 cursor-pointer">
          <input type="checkbox" checked={ruiConfig.server_list.override}
                 onchange={(e) => setRuiField({
                   server_list: { ...ruiConfig.server_list, override: e.currentTarget.checked }
                 })}
                 class="accent-[#f64a46]" />
          <span class="text-[11px] text-white/60">{t("rui.serverListOverride")}</span>
        </label>

        <!-- Actions -->
        <div class="flex flex-wrap items-center gap-2 mt-4">
          <button class={btnPrimary} style="font-family: 'Proxima Nova Semibold', sans-serif;"
                  onclick={applyRuiConfig} onmouseenter={playHoverSound} disabled={ruiBusy}>
            {t("rui.apply")}
          </button>
          <button class={btnGhost} onclick={previewRuiConfig} onmouseenter={playHoverSound} disabled={ruiBusy}>
            {t("rui.preview")}
          </button>
          <button class={btnGhost} onclick={restoreRui} onmouseenter={playHoverSound} disabled={ruiBusy}>
            {t("rui.restore")}
          </button>
        </div>

        {#if ruiError}
          <p class="text-[10px] text-[#f64a46]/80 mt-3">{ruiError}</p>
        {/if}

        {#if ruiLastReport}
          <p class="text-[10px] text-white/40 mt-3">
            {ruiLastReport.entries_changed > 0
              ? tf("rui.applied", String(ruiLastReport.entries_changed))
              : t("rui.noChanges")}
          </p>
        {/if}

        {#if ruiPreview}
          {#if ruiPreview.changes.length}
            <p class="text-[10px] text-white/40 mt-3 mb-1">{t("rui.previewTitle")}</p>
            <ul class="text-[10px] text-white/50 space-y-0.5">
              {#each ruiPreview.changes as c}
                <li>· {c}</li>
              {/each}
            </ul>
          {/if}
          {#if ruiPreview.warnings.length}
            <p class="text-[10px] text-yellow-400/70 mt-2 mb-1">{t("rui.warnings")}</p>
            <ul class="text-[10px] text-yellow-400/60 space-y-0.5">
              {#each ruiPreview.warnings as w}
                <li>· {w}</li>
              {/each}
            </ul>
          {/if}
        {/if}
      {/if}
    {:else}
      <p class="text-[10px] text-white/30">
        {ruiStatus?.reason === "CLIENT_NOT_FOUND" ? t("patches.updaterMissing") : t("misc.notFound")}
      </p>
    {/if}
  </div>

  <!-- About -->
  <div class="settings-card bg-white/5 rounded-lg p-5 stagger-4">
    <h3 class="text-sm font-semibold text-white/70 mb-3" style="font-family: 'Proxima Nova Semibold', sans-serif;">
      {t("settings.aboutLauncher")}
    </h3>
    <p class="text-xs text-white/60">ASTRA Launcher v{updateInfo.current_version || '...'}</p>
    <p class="text-xs text-white/60 mt-1">
      {t("settings.serverAddress")} {serverAddress || "—"}
    </p>
    {#if serverOnline}
      <p class="text-xs text-[#15ff00]/80 mt-1">{t("settings.serverOnline")} — {serverPlayers}/{serverMaxPlayers}</p>
    {:else if serverLoading}
      <p class="text-xs text-yellow-400/70 mt-1">{t("settings.checking")}</p>
    {:else}
      <p class="text-xs text-[#f64a46]/80 mt-1">{t("settings.serverOffline")}</p>
    {/if}
    <button class="mt-2 px-3 py-1 bg-white/5 hover:bg-white/10 rounded text-xs text-white/40 hover:text-white/60 transition-colors border border-white/5 btn-ripple"
            onclick={refreshServerStatus} onmouseenter={playHoverSound} disabled={serverLoading}>
      {serverLoading ? t("settings.refreshing") : t("settings.refreshStatus")}
    </button>
    {#if updateAvailable}
      <div class="mt-3 p-3 rounded-lg bg-[#f64a46]/10 border border-[#f64a46]/20">
        <p class="text-xs text-[#ff8c8c] mb-2">
          🔄 {t("settings.updateAvailable")} v{updateInfo.latest_version}
        </p>
        <button class="px-4 py-1.5 bg-[#f64a46]/20 hover:bg-[#f64a46]/30 rounded text-xs text-[#ff8c8c] transition-colors btn-ripple"
                onclick={openUpdateModal} onmouseenter={playHoverSound}>
          {t("settings.downloadInstall")}
        </button>
      </div>
    {:else if updateChecked}
      <p class="text-xs text-green-400/70 mt-1">{t("settings.updateLatest")}</p>
    {/if}
    <button class="mt-3 px-4 py-1.5 bg-white/5 hover:bg-white/10 rounded text-xs text-white/50 hover:text-white/70 transition-colors border border-white/5 btn-ripple"
            onclick={checkUpdates} onmouseenter={playHoverSound} disabled={updateChecking}>
      {updateChecking ? t("settings.checking") : t("settings.checkUpdates")}
    </button>
  </div>
</div>

<style>
  :global(.settings-card) {
    transition: all 0.3s ease;
  }

  :global(.settings-card:hover) {
    background: rgba(255, 255, 255, 0.08);
  }

  /* Keep the panel scrollable — the direct-connect cards made it taller than
     the window, and <main> clips overflow. */
  .settings-scroll {
    scrollbar-width: thin;
    scrollbar-color: rgba(255, 255, 255, 0.15) transparent;
  }

  .settings-scroll::-webkit-scrollbar {
    width: 6px;
  }

  .settings-scroll::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.15);
    border-radius: 3px;
  }

  .settings-scroll::-webkit-scrollbar-track {
    background: transparent;
  }
</style>
