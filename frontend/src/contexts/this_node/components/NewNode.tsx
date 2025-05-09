import { Text, Input, VStack } from "@chakra-ui/react"
import { useForm } from "react-hook-form"

import { Field, Button, FormActions } from "../../../components"

export interface NewNodeData {
  name: string
}

export type SubmitNewNodeFunc = (data: NewNodeData) => void

export default function NewNode({
  onSubmitNewNode,
}: {
  onSubmitNewNode: SubmitNewNodeFunc
}) {
  const {
    register,
    handleSubmit,
    formState: { errors, isSubmitting },
  } = useForm<NewNodeData>()

  return (
    <VStack alignItems={"stretch"} width="100%">
      <VStack alignItems={"flex-start"} textStyle={"xl"} gap={4}>
        <Text>
          Welcome to your new Node - part of a local, resilient, internet.
        </Text>
        <Text>To get setup, you'll need to choose a node name.</Text>
        <Text>
          Ideally this should be unique within your region, but don't worry,
          you'll have a chance to change it later.
        </Text>
      </VStack>

      <form onSubmit={handleSubmit(onSubmitNewNode)}>
        <VStack alignItems={"flex-start"} pt={8}>
          <Field
            label="Node Name"
            helperText={`A name to identify your Node - use lowercase letters and no spaces`}
            invalid={!!errors.name}
            errorText={errors.name?.message}
          >
            <Input
              {...register("name", {
                required: "This is required",
                maxLength: {
                  value: 50,
                  message: "Must be less than 50 characters",
                },
                pattern: {
                  value: /^[a-z]+(-[a-z]+)*$/,
                  message: "Lowercase letters only, no spaces, hyphens allowed",
                },
              })}
            />
          </Field>

          <FormActions>
            <Button loading={isSubmitting} type="submit">
              Set Name
            </Button>
          </FormActions>
        </VStack>
      </form>
    </VStack>
  )
}
