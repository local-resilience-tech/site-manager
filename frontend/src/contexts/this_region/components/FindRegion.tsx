import { VStack, Text, Tabs, Heading } from "@chakra-ui/react"
import { LuMapPinHouse, LuMapPinPlus } from "react-icons/lu"
import NewRegion, { SubmitNewRegionFunc } from "./NewRegion"

export default function FindRegion({
  onSubmitNewRegion,
}: {
  onSubmitNewRegion: SubmitNewRegionFunc
}) {
  return (
    <VStack alignItems={"stretch"}>
      <Heading as="h1" size="2xl">Welcome to Site Manager</Heading>
      <Text>
        In order to setup this Site, we need to connect you to a region.
      </Text>
      <Text>
        A Region is a cluster of Sites that are in regular communication, and
        provide services to users that are redundantly available at many or all
        of the Sites. This means that a region is generally a geographic area
        that makes sense to humans, such as your neighbourhood, town, river
        catchment, etc.
      </Text>
      <Text>
        You can create a new Region, or join an existing one. What would you
        like to do?
      </Text>
      <Tabs.Root defaultValue="members" variant="line" mt={4}>
        <Tabs.List>
          <Tabs.Trigger value="members">
            <LuMapPinPlus />
            New Region
          </Tabs.Trigger>
          <Tabs.Trigger value="projects">
            <LuMapPinHouse />
            Join Region
          </Tabs.Trigger>
        </Tabs.List>
        <Tabs.Content value="members">
          <NewRegion onSubmitNewRegion={onSubmitNewRegion} />
        </Tabs.Content>
        <Tabs.Content value="projects">
          Todo: Form to join an existing region.
        </Tabs.Content>
      </Tabs.Root>
    </VStack>
  )
}
