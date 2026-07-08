// DaisyUI 5 wireframe blocks. Keep markup minimal — the canvas already
// loads daisyui.css + tailwindcss/browser, so class names alone render.
// Grouped by DaisyUI component category. Exclusions are documented at the
// bottom of this file — anything not shown here would be invisible to the
// user, so if it's usable in a wireframe, it belongs in this list.
const rawBlocks = [
  // ─── Actions ────────────────────────────────────────────────────────────
  {
    id: 'daisy-btn',
    label: 'Button',
    category: 'Actions',
    content: '<button class="btn btn-primary">Button</button>',
  },
  {
    id: 'daisy-btn-group',
    label: 'Button group',
    category: 'Actions',
    content: `<div class="join">
  <button class="btn join-item">Left</button>
  <button class="btn join-item">Middle</button>
  <button class="btn join-item">Right</button>
</div>`,
  },
  {
    id: 'daisy-dropdown',
    label: 'Dropdown',
    category: 'Actions',
    content: `<div class="dropdown">
  <div tabindex="0" role="button" class="btn m-1">Menu</div>
  <ul tabindex="0" class="dropdown-content menu bg-base-100 rounded-box z-1 w-52 p-2 shadow-sm">
    <li><a>Item 1</a></li>
    <li><a>Item 2</a></li>
  </ul>
</div>`,
  },
  {
    id: 'daisy-modal',
    label: 'Modal',
    category: 'Actions',
    content: `<div class="modal modal-open">
  <div class="modal-box">
    <h3 class="text-lg font-bold">Modal title</h3>
    <p class="py-4">Modal body text.</p>
    <div class="modal-action">
      <button class="btn">Close</button>
    </div>
  </div>
</div>`,
  },
  {
    id: 'daisy-swap',
    label: 'Swap',
    category: 'Actions',
    content: `<label class="swap">
  <input type="checkbox" />
  <div class="swap-on">ON</div>
  <div class="swap-off">OFF</div>
</label>`,
  },
  {
    id: 'daisy-theme-controller',
    label: 'Theme controller',
    category: 'Actions',
    content: `<label class="label cursor-pointer gap-2">
  <span class="label-text">Dark mode</span>
  <input type="checkbox" value="dark" class="toggle theme-controller" />
</label>`,
  },

  // ─── Data display ───────────────────────────────────────────────────────
  {
    id: 'daisy-accordion',
    label: 'Accordion',
    category: 'Data display',
    content: `<div class="join join-vertical w-full">
  <div class="collapse collapse-arrow join-item border border-base-300">
    <input type="radio" name="accordion-1" checked />
    <div class="collapse-title font-semibold">Section 1</div>
    <div class="collapse-content"><p>Content 1</p></div>
  </div>
  <div class="collapse collapse-arrow join-item border border-base-300">
    <input type="radio" name="accordion-1" />
    <div class="collapse-title font-semibold">Section 2</div>
    <div class="collapse-content"><p>Content 2</p></div>
  </div>
</div>`,
  },
  {
    id: 'daisy-avatar',
    label: 'Avatar',
    category: 'Data display',
    content: `<div class="avatar">
  <div class="w-16 rounded-full bg-neutral"></div>
</div>`,
  },
  {
    id: 'daisy-badge',
    label: 'Badge',
    category: 'Data display',
    content: '<span class="badge badge-secondary">Badge</span>',
  },
  {
    id: 'daisy-card',
    label: 'Card',
    category: 'Data display',
    content: `<div class="card w-96 bg-base-100 shadow-xl">
  <div class="card-body">
    <h2 class="card-title">Card title</h2>
    <p>Card body text.</p>
    <div class="card-actions justify-end">
      <button class="btn btn-primary">Action</button>
    </div>
  </div>
</div>`,
  },
  {
    id: 'daisy-carousel',
    label: 'Carousel',
    category: 'Data display',
    content: `<div class="carousel rounded-box w-96">
  <div class="carousel-item w-full bg-base-200 h-32 flex items-center justify-center">Slide 1</div>
  <div class="carousel-item w-full bg-base-300 h-32 flex items-center justify-center">Slide 2</div>
</div>`,
  },
  {
    id: 'daisy-chat',
    label: 'Chat bubble',
    category: 'Data display',
    content: `<div class="chat chat-start">
  <div class="chat-bubble">Hello!</div>
</div>
<div class="chat chat-end">
  <div class="chat-bubble chat-bubble-primary">Hi there.</div>
</div>`,
  },
  {
    id: 'daisy-collapse',
    label: 'Collapse',
    category: 'Data display',
    content: `<div class="collapse bg-base-200">
  <input type="checkbox" />
  <div class="collapse-title font-semibold">Click to expand</div>
  <div class="collapse-content"><p>Hidden content.</p></div>
</div>`,
  },
  {
    id: 'daisy-countdown',
    label: 'Countdown',
    category: 'Data display',
    content: `<span class="countdown font-mono text-4xl">
  <span style="--value:12;">12</span>:
  <span style="--value:34;">34</span>:
  <span style="--value:56;">56</span>
</span>`,
  },
  {
    id: 'daisy-diff',
    label: 'Diff',
    category: 'Data display',
    content: `<div class="diff aspect-16/9">
  <div class="diff-item-1"><div class="bg-primary text-primary-content grid place-content-center text-4xl">DAISY</div></div>
  <div class="diff-item-2"><div class="bg-base-200 grid place-content-center text-4xl">daisy</div></div>
  <div class="diff-resizer"></div>
</div>`,
  },
  {
    id: 'daisy-kbd',
    label: 'Keyboard',
    category: 'Data display',
    content: '<kbd class="kbd">Ctrl</kbd> + <kbd class="kbd">K</kbd>',
  },
  {
    id: 'daisy-stats',
    label: 'Stats',
    category: 'Data display',
    content: `<div class="stats shadow">
  <div class="stat">
    <div class="stat-title">Total</div>
    <div class="stat-value">31K</div>
    <div class="stat-desc">Jan 1st - Feb 1st</div>
  </div>
</div>`,
  },
  {
    id: 'daisy-status',
    label: 'Status',
    category: 'Data display',
    content: '<span class="status status-success"></span> Online',
  },
  {
    id: 'daisy-table',
    label: 'Table',
    category: 'Data display',
    content: `<div class="overflow-x-auto">
  <table class="table">
    <thead><tr><th>Name</th><th>Job</th><th>Status</th></tr></thead>
    <tbody>
      <tr><td>Alice</td><td>Designer</td><td>Active</td></tr>
      <tr><td>Bob</td><td>Engineer</td><td>Away</td></tr>
    </tbody>
  </table>
</div>`,
  },
  {
    id: 'daisy-timeline',
    label: 'Timeline',
    category: 'Data display',
    content: `<ul class="timeline">
  <li>
    <div class="timeline-start">1984</div>
    <div class="timeline-middle"></div>
    <div class="timeline-end timeline-box">First event</div>
    <hr />
  </li>
  <li>
    <hr />
    <div class="timeline-start">1998</div>
    <div class="timeline-middle"></div>
    <div class="timeline-end timeline-box">Second event</div>
  </li>
</ul>`,
  },

  // ─── Data input ─────────────────────────────────────────────────────────
  {
    id: 'daisy-input',
    label: 'Input',
    category: 'Data input',
    content: '<input type="text" placeholder="Type here" class="input input-bordered w-full max-w-xs" />',
  },
  {
    id: 'daisy-textarea',
    label: 'Textarea',
    category: 'Data input',
    content: '<textarea class="textarea textarea-bordered w-full max-w-xs" placeholder="Bio"></textarea>',
  },
  {
    id: 'daisy-select',
    label: 'Select',
    category: 'Data input',
    content: `<select class="select select-bordered w-full max-w-xs">
  <option disabled selected>Pick one</option>
  <option>Option A</option>
  <option>Option B</option>
</select>`,
  },
  {
    id: 'daisy-checkbox',
    label: 'Checkbox',
    category: 'Data input',
    content: `<label class="label cursor-pointer gap-2">
  <input type="checkbox" class="checkbox checkbox-primary" />
  <span class="label-text">Remember me</span>
</label>`,
  },
  {
    id: 'daisy-radio',
    label: 'Radio',
    category: 'Data input',
    content: `<label class="label cursor-pointer gap-2">
  <input type="radio" name="radio-1" class="radio radio-primary" checked />
  <span class="label-text">Choice A</span>
</label>`,
  },
  {
    id: 'daisy-toggle',
    label: 'Toggle',
    category: 'Data input',
    content: '<input type="checkbox" class="toggle toggle-primary" checked />',
  },
  {
    id: 'daisy-range',
    label: 'Range',
    category: 'Data input',
    content: '<input type="range" min="0" max="100" value="40" class="range range-primary" />',
  },
  {
    id: 'daisy-file-input',
    label: 'File input',
    category: 'Data input',
    content: '<input type="file" class="file-input file-input-bordered w-full max-w-xs" />',
  },
  {
    id: 'daisy-rating',
    label: 'Rating',
    category: 'Data input',
    content: `<div class="rating">
  <input type="radio" name="rating-1" class="mask mask-star" />
  <input type="radio" name="rating-1" class="mask mask-star" checked />
  <input type="radio" name="rating-1" class="mask mask-star" />
  <input type="radio" name="rating-1" class="mask mask-star" />
  <input type="radio" name="rating-1" class="mask mask-star" />
</div>`,
  },
  {
    id: 'daisy-fieldset',
    label: 'Fieldset',
    category: 'Data input',
    content: `<fieldset class="fieldset bg-base-200 border border-base-300 rounded-box w-xs p-4">
  <legend class="fieldset-legend">Page details</legend>
  <label class="fieldset-label">Title</label>
  <input type="text" class="input" placeholder="My awesome page" />
</fieldset>`,
  },

  // ─── Feedback ───────────────────────────────────────────────────────────
  {
    id: 'daisy-alert',
    label: 'Alert',
    category: 'Feedback',
    content: '<div class="alert alert-info"><span>Info message</span></div>',
  },
  {
    id: 'daisy-progress',
    label: 'Progress',
    category: 'Feedback',
    content: '<progress class="progress progress-primary w-56" value="40" max="100"></progress>',
  },
  {
    id: 'daisy-radial-progress',
    label: 'Radial progress',
    category: 'Feedback',
    content: '<div class="radial-progress text-primary" style="--value:70;">70%</div>',
  },
  {
    id: 'daisy-loading',
    label: 'Loading',
    category: 'Feedback',
    content: '<span class="loading loading-spinner loading-lg"></span>',
  },
  {
    id: 'daisy-skeleton',
    label: 'Skeleton',
    category: 'Feedback',
    content: '<div class="skeleton h-32 w-full"></div>',
  },
  {
    id: 'daisy-tooltip',
    label: 'Tooltip',
    category: 'Feedback',
    content: `<div class="tooltip tooltip-open" data-tip="hello">
  <button class="btn">Hover me</button>
</div>`,
  },
  {
    id: 'daisy-toast',
    label: 'Toast',
    category: 'Feedback',
    content: `<div class="toast">
  <div class="alert alert-info"><span>New message</span></div>
</div>`,
  },

  // ─── Layout ─────────────────────────────────────────────────────────────
  {
    id: 'daisy-divider',
    label: 'Divider',
    category: 'Layout',
    content: '<div class="divider">OR</div>',
  },
  {
    id: 'daisy-drawer',
    label: 'Drawer',
    category: 'Layout',
    content: `<div class="drawer">
  <input id="drawer-1" type="checkbox" class="drawer-toggle" />
  <div class="drawer-content p-4">
    <label for="drawer-1" class="btn btn-primary drawer-button">Open drawer</label>
  </div>
  <div class="drawer-side">
    <label for="drawer-1" class="drawer-overlay"></label>
    <ul class="menu bg-base-200 min-h-full w-64 p-4">
      <li><a>Sidebar item 1</a></li>
      <li><a>Sidebar item 2</a></li>
    </ul>
  </div>
</div>`,
  },
  {
    id: 'daisy-footer',
    label: 'Footer',
    category: 'Layout',
    content: `<footer class="footer bg-neutral text-neutral-content p-10">
  <nav>
    <h6 class="footer-title">Services</h6>
    <a class="link link-hover">Branding</a>
    <a class="link link-hover">Design</a>
  </nav>
  <nav>
    <h6 class="footer-title">Company</h6>
    <a class="link link-hover">About us</a>
    <a class="link link-hover">Contact</a>
  </nav>
</footer>`,
  },
  {
    id: 'daisy-hero',
    label: 'Hero',
    category: 'Layout',
    content: `<div class="hero bg-base-200 py-16">
  <div class="hero-content text-center">
    <div class="max-w-md">
      <h1 class="text-4xl font-bold">Hello there</h1>
      <p class="py-6">Hero section body copy goes here.</p>
      <button class="btn btn-primary">Get Started</button>
    </div>
  </div>
</div>`,
  },
  {
    id: 'daisy-indicator',
    label: 'Indicator',
    category: 'Layout',
    content: `<div class="indicator">
  <span class="indicator-item badge badge-secondary">new</span>
  <button class="btn">Inbox</button>
</div>`,
  },
  {
    id: 'daisy-stack',
    label: 'Stack',
    category: 'Layout',
    content: `<div class="stack">
  <div class="grid w-32 h-20 rounded bg-primary text-primary-content place-content-center">1</div>
  <div class="grid w-32 h-20 rounded bg-accent text-accent-content place-content-center">2</div>
  <div class="grid w-32 h-20 rounded bg-secondary text-secondary-content place-content-center">3</div>
</div>`,
  },

  // ─── Mockup ─────────────────────────────────────────────────────────────
  {
    id: 'daisy-mockup-browser',
    label: 'Browser mockup',
    category: 'Mockup',
    content: `<div class="mockup-browser border border-base-300 w-96">
  <div class="mockup-browser-toolbar"><div class="input">https://example.com</div></div>
  <div class="bg-base-200 flex justify-center px-4 py-16">Page content</div>
</div>`,
  },
  {
    id: 'daisy-mockup-code',
    label: 'Code mockup',
    category: 'Mockup',
    content: `<div class="mockup-code w-96">
  <pre data-prefix="$"><code>npm install</code></pre>
  <pre data-prefix=">" class="text-success"><code>Done!</code></pre>
</div>`,
  },
  {
    id: 'daisy-mockup-phone',
    label: 'Phone mockup',
    category: 'Mockup',
    content: `<div class="mockup-phone">
  <div class="mockup-phone-camera"></div>
  <div class="mockup-phone-display bg-base-200 grid place-content-center">Phone content</div>
</div>`,
  },
  {
    id: 'daisy-mockup-window',
    label: 'Window mockup',
    category: 'Mockup',
    content: `<div class="mockup-window border border-base-300 w-96">
  <div class="bg-base-200 flex justify-center px-4 py-16">Window content</div>
</div>`,
  },

  // ─── Navigation ─────────────────────────────────────────────────────────
  {
    id: 'daisy-navbar',
    label: 'Navbar',
    category: 'Navigation',
    content: `<div class="navbar bg-base-100 shadow">
  <div class="flex-1"><a class="btn btn-ghost text-xl">brand</a></div>
  <div class="flex-none">
    <button class="btn btn-ghost">Menu</button>
  </div>
</div>`,
  },
  {
    id: 'daisy-breadcrumbs',
    label: 'Breadcrumbs',
    category: 'Navigation',
    content: `<div class="breadcrumbs text-sm">
  <ul>
    <li><a>Home</a></li>
    <li><a>Docs</a></li>
    <li>Add doc</li>
  </ul>
</div>`,
  },
  {
    id: 'daisy-dock',
    label: 'Dock',
    category: 'Navigation',
    content: `<div class="dock">
  <button><span class="dock-label">Home</span></button>
  <button class="dock-active"><span class="dock-label">Inbox</span></button>
  <button><span class="dock-label">Settings</span></button>
</div>`,
  },
  {
    id: 'daisy-link',
    label: 'Link',
    category: 'Navigation',
    content: '<a class="link link-primary">Anchor link</a>',
  },
  {
    id: 'daisy-menu',
    label: 'Menu',
    category: 'Navigation',
    content: `<ul class="menu bg-base-200 rounded-box w-56">
  <li><a>Item 1</a></li>
  <li><a>Item 2</a></li>
  <li><a>Item 3</a></li>
</ul>`,
  },
  {
    id: 'daisy-tabs',
    label: 'Tabs',
    category: 'Navigation',
    content: `<div role="tablist" class="tabs tabs-bordered">
  <a role="tab" class="tab">Tab 1</a>
  <a role="tab" class="tab tab-active">Tab 2</a>
  <a role="tab" class="tab">Tab 3</a>
</div>`,
  },
  {
    id: 'daisy-steps',
    label: 'Steps',
    category: 'Navigation',
    content: `<ul class="steps">
  <li class="step step-primary">Register</li>
  <li class="step step-primary">Choose plan</li>
  <li class="step">Purchase</li>
  <li class="step">Receive</li>
</ul>`,
  },
  {
    id: 'daisy-pagination',
    label: 'Pagination',
    category: 'Navigation',
    content: `<div class="join">
  <button class="join-item btn">«</button>
  <button class="join-item btn">Page 1</button>
  <button class="join-item btn btn-active">Page 2</button>
  <button class="join-item btn">Page 3</button>
  <button class="join-item btn">»</button>
</div>`,
  },
];

// Reuse each block's actual DaisyUI markup as its Blocks-panel thumbnail
// (Figma-style component library). Scaling is done via CSS on the wrapper.
// Some components render as blank when closed (Modal <dialog>, Drawer checkbox
// off, Toast off-screen fixed). Force an "open/visible" variant for the
// thumbnail only — canvas content stays canonical.
const mediaOverrides = {
  'daisy-modal': `<div class="modal modal-open" style="position:static">
  <div class="modal-box"><h3 class="font-bold text-lg">Modal title</h3><p class="py-4">Modal body</p></div>
</div>`,
  'daisy-toast': `<div class="toast toast-top toast-end" style="position:static">
  <div class="alert alert-info"><span>New message</span></div>
</div>`,
  'daisy-drawer': `<div class="drawer drawer-open">
  <div class="drawer-content p-2"><span class="btn btn-primary btn-sm">Content</span></div>
  <div class="drawer-side">
    <ul class="menu bg-base-200 w-40 p-2 text-xs">
      <li><a>Sidebar 1</a></li>
      <li><a>Sidebar 2</a></li>
    </ul>
  </div>
</div>`,
};

export const daisyBlocks = rawBlocks.map((b) => {
  const preview = mediaOverrides[b.id] ?? b.content;
  return { ...b, media: `<div class="wf-block-preview">${preview}</div>` };
});

// ─── Documented exclusions ────────────────────────────────────────────────
// The following DaisyUI 5 components are intentionally NOT surfaced as
// blocks. Rationale is kept here so future maintainers do not "restore" them
// without cause.
//
// - Join, Mask: helper utilities that only make sense combined with another
//   component (Button group / Pagination / Rating already showcase them).
// - Artboard: legacy mobile mockup helper superseded by mockup-phone.
// - Calendar, Filter, Validator: require host-app JS state / third-party
//   libs (cally, filter reset, form validation); they render nothing useful
//   without wiring, so they'd be misleading in a static wireframe canvas.
// - Label: layout helper always used inside another form component
//   (Checkbox / Radio blocks already demonstrate its use).
