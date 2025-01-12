import { RouterProvider, createBrowserRouter } from "react-router-dom"
import Layout from "./pages/Layout"
import { ChakraProvider } from "@chakra-ui/react"
import { ColorModeProvider } from "./components/ui/color-mode"
import { InstalledApps } from "./contexts/apps"
import { themeSystem } from "./theme"
import { ThisSite, ThisNode } from "./contexts/this_site"
import { ThisRegion } from "./contexts/this_region"

const router = createBrowserRouter(
  [
    {
      path: "/",
      element: <Layout />,
      children: [
        { path: "", element: <ThisRegion /> },
        { path: "node", element: <ThisNode /> },
        { path: "apps", element: <InstalledApps /> },
      ],
    },
  ],
  {
    basename: "/admin",
  },
)

function App() {
  return (
    <ChakraProvider value={themeSystem}>
      <ColorModeProvider>
        <RouterProvider router={router} />
      </ColorModeProvider>
    </ChakraProvider>
  )
}

export default App
