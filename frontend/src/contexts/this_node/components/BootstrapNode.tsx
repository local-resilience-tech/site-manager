import { Input, Box, Text } from "@chakra-ui/react"
import { Field, FormActions, Button, FormFields } from "../../../components"
import { useForm } from "react-hook-form"

export interface BootstrapNodeData {
  network_name: string
  node_id: string
  ip_address: string
}

export type SubmitBootstrapNodeFunc = (data: BootstrapNodeData) => void

export default function BootstrapNode({
  onSubmit,
}: {
  onSubmit: SubmitBootstrapNodeFunc
}) {
  const {
    register,
    handleSubmit,
    formState: { errors, isSubmitting },
  } = useForm<BootstrapNodeData>()

  return (
    <form onSubmit={handleSubmit(onSubmit)}>
      <Text as="p" mb={4}>
        TODO: We don't yet provide much feedback on whether you put in the
        correct details here, please type carefully.
      </Text>
      <FormFields>
        <Field
          label="Region Name"
          helperText={`A unique string that defines this region`}
          invalid={!!errors.network_name}
          errorText={errors.network_name?.message}
        >
          <Input
            {...register("network_name", {
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
        <Field
          label="Node ID"
          helperText={`A hex string that identifies another node in this network`}
          invalid={!!errors.node_id}
          errorText={errors.node_id?.message}
        >
          <Input
            {...register("node_id", {
              required: "This is required",
              maxLength: {
                value: 50,
                message: "Must be less than 50 characters",
              },
            })}
          />
        </Field>
        <Field
          label="Public IP Address"
          helperText={`An IPv4 address that can be used to communicate with the node above`}
          invalid={!!errors.ip_address}
          errorText={errors.ip_address?.message}
        >
          <Input
            {...register("ip_address", {
              required: "This is required",
              pattern: {
                value: /^(\d{1,3}\.){3}\d{1,3}$/,
                message: "Must be a valid IPv4 address",
              },
            })}
          />
        </Field>
      </FormFields>
      <FormActions>
        <Button loading={isSubmitting} type="submit">
          Connect
        </Button>
      </FormActions>
    </form>
  )
}
