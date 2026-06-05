<!--
  Background.svelte — Анимированный фон (частицы + градиент + декоративные SVG)
  Принимает parallaxX, parallaxY, dpiScale, particleStyles
-->
<script>
  let { parallaxX = 0, parallaxY = 0, dpiScale = 1, particleStyles = [] } = $props();
</script>

<!-- Плавающие градиентные блобы -->
<div class="absolute inset-0 z-0 overflow-hidden pointer-events-none transition-transform duration-300 ease-out"
     style="transform: translate({parallaxX * 0.3}px, {parallaxY * 0.3}px);">
  <div class="animated-blob blob-1"></div>
  <div class="animated-blob blob-2"></div>
  <div class="animated-blob blob-3"></div>
  <!-- Частицы (звёзды) -->
  {#each particleStyles as ps, i}
    <div class="particle" style="
      left: {ps.left};
      top: {ps.top};
      animation-delay: {ps.delay};
      animation-duration: {ps.duration};
      width: {ps.width};
      height: {ps.height};
    "></div>
  {/each}
</div>

<!-- Subtract1 — красная подсветка справа сверху -->
<svg class="absolute top-0 left-[504px] w-[456px] h-[400px] pointer-events-none select-none z-[6]"
     viewBox="0 0 456 400" fill="none">
  <defs><filter id="glow1" x="-200" y="-200" width="900" height="900" filterUnits="userSpaceOnUse">
    <feGaussianBlur stdDeviation="100"/><feComposite in2="hardAlpha" operator="out"/>
    <feColorMatrix type="matrix" values="0 0 0 0 0.965 0 0 0 0 0.29 0 0 0 0 0.275 0 0 0 1 0"/>
    <feBlend mode="normal" in2="BackgroundImageFix" result="effect1"/><feBlend mode="normal" in="SourceGraphic" in2="effect1" result="shape"/>
  </filter></defs>
  <g filter="url(#glow1)"><path d="M367.5 200C460 200 535 125 535 32.5C535 -60 460 -135 367.5 -135C275 -135 200 -60 200 32.5C200 125 275 200 367.5 200ZM368 164.5C295.6 164.5 236.8 105.7 236.8 33.2C236.8 -39.4 295.6 -98.2 368 -98.2C440.7 -98.2 499.5 -39.4 499.5 33.2C499.5 105.7 440.7 164.5 368 164.5Z" fill="#F64A46"/></g>
</svg>

<!-- Subtract — красная подсветка слева снизу -->
<svg class="absolute top-[282px] left-0 w-[642px] h-[358px] pointer-events-none select-none z-[6]"
     viewBox="0 0 647 386" fill="none">
  <defs><filter id="glow2" x="-200" y="-200" width="1100" height="900" filterUnits="userSpaceOnUse">
    <feGaussianBlur stdDeviation="100"/><feComposite in2="hardAlpha" operator="out"/>
    <feColorMatrix type="matrix" values="0 0 0 0 0.965 0 0 0 0 0.29 0 0 0 0 0.275 0 0 0 1 0"/>
    <feBlend mode="normal" in2="BackgroundImageFix" result="effect1"/><feBlend mode="normal" in="SourceGraphic" in2="effect1" result="shape"/>
  </filter></defs>
  <g filter="url(#glow2)"><path d="M279.5 535C372 535 447 460 447 367.5C447 275 372 200 279.5 200C187 200 112 275 112 367.5C112 460 187 535 279.5 535ZM280 499.5C207.6 499.5 148.8 440.7 148.8 368.2C148.8 295.6 207.6 236.8 280 236.8C352.7 236.8 411.5 295.6 411.5 368.2C411.5 440.7 352.7 499.5 280 499.5Z" fill="#F64A46"/></g>
</svg>

<!-- GTAV персонаж — выше текста ASTRA и звёзд, но ниже кнопки играть (параллакс) -->
<img src="/GTAV-33-4.png.png" alt="" class="absolute top-4 left-[16.5%] w-[83.5%] h-[104%] object-cover pointer-events-none select-none z-[5] transition-transform duration-300 ease-out"
     style="transform: translate({parallaxX * 0.8}px, {parallaxY * 0.8}px);" aria-hidden="true" />

<!-- Большой текст ASTRA на фоне (400px как в imdex.tsx) -->
<div class="pointer-events-none select-none"
     style="position: absolute; top: {108 / dpiScale}px; left: {230 / dpiScale}px;">
  <div class="pointer-events-none select-none"
       style="font-family: 'Armor Piercing 2.0 BB', 'Impact', sans-serif; font-size: {400 / dpiScale}px; font-weight: normal; color: transparent; -webkit-text-stroke: 1px rgba(255,255,255,0.05); letter-spacing: {-8 / dpiScale}px; line-height: normal; white-space: nowrap;">
    ASTRA
  </div>
</div>

<!-- Star 23 — звезда справа снизу (параллакс) -->
<svg class="absolute top-[410px] left-[702px] w-[258px] h-[230px] pointer-events-none select-none transition-transform duration-300 ease-out"
     style="transform: translate({parallaxX * 1.2}px, {parallaxY * 1.2}px);" viewBox="0 0 259 230" fill="none">
  <path d="M95.1 301.5C94.06 325 123.66 336.1 138.3 317.7L182.8 261.8C189.2 253.8 200.1 250.8 209.7 254.4L276.6 279.5C298.6 287.7 318.3 263 305.4 243.4L266 183.8C260.3 175.2 260.8 164 267.2 155.9L311.8 100C326.4 81.7 309 55.3 286.3 61.5L217.5 80.6C207.5 83.3 197 79.4 191.3 70.8L151.9 11.2C139 -8.4 108.5 0 107.5 23.5L104.3 94.8C103.8 105.1 96.8 113.9 86.9 116.7L18 135.7C-4.6 142 -6 173.5 16 181.8L82.9 206.9C92.5 210.5 98.7 219.9 98.3 230.2L95.1 301.5Z" stroke="white" stroke-opacity="0.3" stroke-width="0.5"/>
</svg>

<!-- Star 24 — звезда сверху (параллакс) -->
<svg class="absolute top-0 left-[106px] w-[323px] h-[156px] pointer-events-none select-none transition-transform duration-300 ease-out"
     style="transform: translate({parallaxX * 1.4}px, {parallaxY * 1.4}px);" viewBox="0 0 324 157" fill="none">
  <path d="M201.2 148.6C217.6 165.4 246 151.5 242.6 128.2L232.5 57.5C231 47.3 236.2 37.3 245.5 32.8L309.6 1.3C330.6 -9.1 326.1 -40.4 303 -44.4L232.6 -56.6C222.5 -58.3 214.6 -66.4 213.1 -76.6L202.9 -147.3C199.6 -170.5 168.5 -175.9 157.5 -155.1L124.2 -91.9C119.4 -82.8 109.3 -77.9 99.1 -79.6L28.7 -91.8C5.6 -95.8 -9.2 -67.8 7.2 -51L57 0.2C64.2 7.6 65.8 18.8 61 27.9L27.7 91.1C16.7 111.9 38.7 134.5 59.8 124.1L123.9 92.6C133.2 88.1 144.3 90 151.4 97.4L201.2 148.6Z" stroke="white" stroke-opacity="0.3" stroke-width="0.5"/>
</svg>
