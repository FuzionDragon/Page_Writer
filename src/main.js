import { invoke } from '@tauri-apps/api/core';

window.onkeyup = function(e){
  if(e.ctrlKey && e.key === 'Enter'){
    invoke('print_test');
  }
}
