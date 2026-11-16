export default defineAppConfig({
  ui: {
    colors: {
      primary: 'orange',
      secondary: 'amber',
      neutral: 'slate',
    },
    modal: {
      slots: {
        content: 'bg-transparent ring-0 shadow-none border-0 divide-y-0',
      },
      variants: {
        fullscreen: {
          false: {
            content: 'w-[calc(100vw-2rem)] ring-0 shadow-none',
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
