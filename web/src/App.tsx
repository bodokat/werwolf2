import { useState } from 'react'
import reactLogo from './assets/react.svg'
import './App.css'
import { createBrowserRouter, RouterProvider } from 'react-router-dom'
import { MantineProvider, Text } from '@mantine/core';



const router = createBrowserRouter([
  {
    path: '/',
    element: <Text>Welcome</Text>
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
