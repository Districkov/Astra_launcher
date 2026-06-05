<!--
  Notifications.svelte — Toast-уведомления
  Принимает массив notifications
-->
<script>
  let { notifications = [] } = $props();
</script>

<div class="absolute top-[40px] right-[16px] z-[200] flex flex-col gap-2 pointer-events-none">
  {#each notifications as notif (notif.id)}
    <div class="pointer-events-auto px-4 py-2.5 rounded-lg text-sm shadow-lg backdrop-blur-sm transition-all duration-300 animate-slide-in
      {notif.type === 'success' ? 'bg-green-500/20 border border-green-500/30 text-green-300' :
       notif.type === 'error' ? 'bg-red-500/20 border border-red-500/30 text-red-300' :
       notif.type === 'update' ? 'bg-[#f64a46]/20 border border-[#f64a46]/30 text-[#ff8c8c]' :
       'bg-white/10 border border-white/10 text-white/70'}">
      {notif.message}
    </div>
  {/each}
</div>
