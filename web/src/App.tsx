import { createBrowserRouter, RouterProvider } from 'react-router-dom'
import { MantineProvider } from '@mantine/core';
import { GameSession } from "./api/gameSession";
import { Login } from "./login";
import { Game } from './game';
import { Notifications } from '@mantine/notifications';



const router = createBrowserRouter([
  {
    path: '/',
    element: <Login />
  }, {
    path: '/l/:lobby',
    loader: (({ params: { lobby } }) => {
      console.log("loading")
      if (typeof lobby == "string") return GameSession.join(lobby)
    }),
    element: <Game />

  }
])

function App() {

  return (
    <MantineProvider withNormalizeCSS withGlobalStyles theme={{ colorScheme: 'dark' }}>
      <Notifications />
      <RouterProvider router={router} />
    </MantineProvider>
  )
}

export default App
