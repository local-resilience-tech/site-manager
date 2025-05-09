import { VStack, Text, Tabs, Heading } from "@chakra-ui/react"
import { LuMapPinHouse, LuMapPinPlus } from "react-icons/lu"
import NewRegion, { SubmitNewRegionFunc } from "./NewRegion"
import ExistingRegion from "./ExistingRegion"

export default function SetRegion({
  onSubmitNewRegion,
}: {
  onSubmitNewRegion: SubmitNewRegionFunc
}) {
  return (
    <VStack alignItems={"stretch"}>
      <Heading as="h1" size="2xl">
        Welcome to LoRes Mesh
      </Heading>
      <Text>
        In order to setup this Node, we need to connect you to a region.
      </Text>
      <Text>
        A Region is a cluster of Nodes that are in regular communication, and
        provide services to users that are redundantly available at many or all
        of the Nodes. This means that a region is generally a geographic area
        that makes sense to humans, such as your neighbourhood, town, river
        catchment, etc.
      </Text>
      <Text>
        You can join an existing region, or create a new one. What would you
        like to do?
      </Text>
      <Tabs.Root defaultValue="join" variant="line" mt={4}>
        <Tabs.List>
          <Tabs.Trigger value="join">
            <LuMapPinHouse />
            Join Region
          </Tabs.Trigger>
          <Tabs.Trigger value="new">
            <LuMapPinPlus />
            New Region
          </Tabs.Trigger>
        </Tabs.List>
        <Tabs.Content value="join">
          <ExistingRegion />
        </Tabs.Content>
        <Tabs.Content value="new">
          <NewRegion onSubmitNewRegion={onSubmitNewRegion} />
        </Tabs.Content>
      </Tabs.Root>
    </VStack>
  )
}
