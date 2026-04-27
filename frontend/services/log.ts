// Development-only logging helpers
export const log = import.meta.env.DEV ? console.log : () => {};
export const logError = import.meta.env.DEV ? console.error : () => {};
