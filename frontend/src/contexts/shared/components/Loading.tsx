import { Container, Center, Spinner } from "@chakra-ui/react"

export default function Loading() {
  return (
    <Container maxWidth={"2xl"}>
      <Center>
        <Spinner size="lg" opacity={0.5} />
      </Center>
    </Container>
  )
}
