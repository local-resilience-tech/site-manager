import { RouterProvider, createBrowserRouter } from "react-router-dom"
import Layout from "./pages/Layout"
import { ChakraProvider } from "@chakra-ui/react"
import { ColorModeProvider } from "./components/ui/color-mode"
import { themeSystem } from "./theme"
import { EnsureSite } from "./contexts/this_site"
import { ThisP2PandaNode } from "./contexts/this_p2panda_node"
import { EnsureRegion, Sites } from "./contexts/this_region"

const router = createBrowserRouter(
  [
    {
      path: "/",
      element: <Layout />,
      children: [
        {
          path: "",
          element: <EnsureRegion />,
          children: [
            { path: "sites", element: <Sites /> },
            { path: "this_site", element: <EnsureSite /> },
          ],
        },
        { path: "node", element: <ThisP2PandaNode /> },
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
