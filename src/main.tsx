import { RouterProvider, createRouter } from '@tanstack/react-router';
import React from 'react';
import ReactDOM from 'react-dom/client';
import { IntlayerProvider } from "react-intlayer";

import { routeTree } from './routeTree.gen';

const router = createRouter({ routeTree });

// Make the router available in the entire app via React context, see https://tanstack.com/router/docs/react/overview
declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <IntlayerProvider>
      <RouterProvider router={router} />
    </IntlayerProvider>
  </React.StrictMode>,
);
