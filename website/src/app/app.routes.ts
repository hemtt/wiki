import { Routes } from '@angular/router';
import { CommandViewerComponent } from './components/command-viewer/command-viewer.component';
import { CommandDetailComponent } from './components/command-detail/command-detail.component';

export const routes: Routes = [
  {
    path: '',
    component: CommandViewerComponent,
  },
  {
    path: 'command/:name',
    component: CommandDetailComponent,
  },
  {
    path: '**',
    redirectTo: '',
  },
];
