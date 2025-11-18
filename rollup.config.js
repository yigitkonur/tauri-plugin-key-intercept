import typescript from '@rollup/plugin-typescript';

export default {
  input: 'guest-js/index.ts',
  output: [
    {
      file: 'dist-js/index.cjs',
      format: 'cjs',
      exports: 'named'
    },
    {
      file: 'dist-js/index.js',
      format: 'esm'
    }
  ],
  plugins: [
    typescript({
      declaration: true,
      declarationDir: 'dist-js',
      rootDir: 'guest-js'
    })
  ],
  external: ['@tauri-apps/api/core', '@tauri-apps/api/event']
};

