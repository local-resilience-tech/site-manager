import { RouterProvider, createBrowserRouter } from "react-router-dom"
import Layout from "./pages/Layout"
import { ChakraProvider } from "@chakra-ui/react"
import { ColorModeProvider } from "./components/ui/color-mode"
import { InstalledApps } from "./contexts/apps"
import { themeSystem } from "./theme"
import { EnsureSite } from "./contexts/this_site"
import { ThisNode } from "./contexts/this_node"
import { EnsureRegion, RegionSites } from "./contexts/this_region"

const router = createBrowserRouter(
  [
    {
      path: "/",
      element: <Layout />,
      children: [
        {
          path: "",
          element: (
            <EnsureRegion>
              <EnsureSite />
            </EnsureRegion>
          ),
        },
        { path: "region", element: <RegionSites /> },
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
