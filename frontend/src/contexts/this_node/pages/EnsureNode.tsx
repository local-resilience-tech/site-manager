import { Container } from "@chakra-ui/react"
import { useEffect, useState } from "react"
import NewNode, { NewNodeData } from "../components/NewNode"
import { NodeDetails } from "../types"
import ThisNodeApi from "../api"
import { ApiResult } from "../../shared/types"
import { Loading, useLoading } from "../../shared"

const api = new ThisNodeApi()

const getNode = async (): Promise<NodeDetails | null> => {
  const result = await api.show()
  if ("Ok" in result) return result.Ok
  return null
}

export default function EnsureNode() {
  const [node, setNode] = useState<NodeDetails | null>(null)
  const [loading, withLoading] = useLoading(true)

  const updateNode = (newNode: NodeDetails | null) => {
    console.log("Updating node", newNode)
    setNode(newNode)
  }

  const fetchNode = async () => {
    withLoading(async () => {
      console.log("EFFECT: fetchNode")
      const newNode = await getNode()
      updateNode(newNode)
    })
  }

  useEffect(() => {
    if (node == null) fetchNode()
  }, [])

  const onSubmitNewNode = (data: NewNodeData) => {
    api.create(data.name).then((result: ApiResult<NodeDetails, any>) => {
      if ("Ok" in result) updateNode(result.Ok)
    })
  }

  if (loading) return <Loading />

  return (
    <Container maxWidth={"2xl"}>
      {node == null && <NewNode onSubmitNewNode={onSubmitNewNode} />}
      {node != null && (
        <div>
          <h1>Node created!</h1>
          <p>Node: {node?.name}</p>
        </div>
      )}
    </Container>
  )
}
