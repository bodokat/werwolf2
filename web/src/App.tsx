import { useState } from 'react'
import reactLogo from './assets/react.svg'
import { createBrowserRouter, RouterProvider } from 'react-router-dom'
import { MantineProvider, Text } from '@mantine/core';
import { GameSession } from "./api/gameSession";
import { Login } from "./login";
import { Game } from './game';



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
      <RouterProvider router={router} />
    </MantineProvider>
  )
}

export default App
