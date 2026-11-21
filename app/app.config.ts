export default defineAppConfig({
  ui: {
    colors: {
      primary: 'orange',
      secondary: 'amber',
      neutral: 'slate',
    },
    modal: {
      slots: {
        overlay: 'fixed inset-0 bg-black/75 backdrop-blur-none transition-opacity',
        content: 'bg-[var(--gv-surface)] border border-[var(--gv-border)] shadow-2xl rounded-2xl overflow-hidden focus:outline-none',
      },
      variants: {
        transition: {
          true: {
            overlay: 'data-[state=open]:opacity-100 data-[state=closed]:opacity-0 transition-opacity duration-150',
            content: 'data-[state=open]:opacity-100 data-[state=closed]:opacity-0 transition-opacity duration-150',
          },
        },
        fullscreen: {
          false: {
            content: 'w-[calc(100vw-2rem)]',
          },
        },
      },
    },
    card: {
      slots: {
        root: 'rounded-xl overflow-hidden bg-[var(--gv-surface)] border border-[var(--gv-border)] shadow-xs divide-y divide-[var(--gv-border)]',
        header: 'p-4 sm:px-5',
        body: 'p-4 sm:p-5',
        footer: 'p-4 sm:px-5',
        title: 'text-highlighted font-semibold',
        description: 'mt-1 text-muted text-xs',
      },
      variants: {
        variant: {
          outline: {
            root: 'bg-[var(--gv-surface)] ring-0 border border-[var(--gv-border)] divide-y divide-[var(--gv-border)]',
          },
          subtle: {
            root: 'bg-[var(--gv-surface)] ring-0 border border-[var(--gv-border)] divide-y divide-[var(--gv-border)]',
          },
          soft: {
            root: 'bg-[var(--gv-surface)] border-0 divide-y divide-[var(--gv-border)]',
          },
        },
      },
    },
  },
})
