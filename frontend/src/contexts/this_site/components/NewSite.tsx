import { Text, Input, VStack } from "@chakra-ui/react"
import { useForm } from "react-hook-form"

import { Field, Button, FormActions } from "../../../components"

export interface NewSiteData {
  name: string
}

export type SubmitNewSiteFunc = (data: NewSiteData) => void

export default function NewSite({
  onSubmitNewSite,
}: {
  onSubmitNewSite: SubmitNewSiteFunc
}) {
  const {
    register,
    handleSubmit,
    formState: { errors, isSubmitting },
  } = useForm<NewSiteData>()

  return (
    <VStack alignItems={"stretch"} width="100%">
      <VStack alignItems={"flex-start"} textStyle={"xl"} gap={4}>
        <Text>
          Welcome to your new Site - part of a local, resilient, internet.
        </Text>
        <Text>To get setup, you'll need to choose a site name.</Text>
        <Text>
          Ideally this should be unique within your region, but don't worry,
          you'll have a chance to change it later.
        </Text>
      </VStack>

      <form onSubmit={handleSubmit(onSubmitNewSite)}>
        <VStack alignItems={"flex-start"} pt={8}>
          <Field
            label="Site Name"
            helperText={`A name to identify your Site - use lowercase letters and no spaces`}
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
