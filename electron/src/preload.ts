// See the Electron documentation for details on how to use preload scripts:
// https://www.electronjs.org/docs/latest/tutorial/process-model#preload-scripts
import { contextBridge, ipcRenderer } from 'electron';

export const electronAPI = {
  setTitle: (title: string) => ipcRenderer.send('set-title', title),
};


process.once('loaded', () => {
  contextBridge.exposeInMainWorld('electronAPI', electronAPI);
});