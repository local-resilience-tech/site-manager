import { Container, Center, Spinner } from "@chakra-ui/react"
import { useState } from "react"

type WithLoadingType = (fn: () => Promise<void>) => Promise<void>

export const useLoading = (
  initialState: boolean = true,
): [loading: boolean, withLoading: WithLoadingType] => {
  const [loading, setLoading] = useState(initialState)

  const withLoading: WithLoadingType = async (fn: () => Promise<void>) => {
    setLoading(true)
    await fn()
    setLoading(false)
  }

  return [loading, withLoading]
}

export default function Loading() {
  return (
    <Container maxWidth={"2xl"}>
      <Center>
        <Spinner size="lg" opacity={0.5} />
      </Center>
    </Container>
  )
}
