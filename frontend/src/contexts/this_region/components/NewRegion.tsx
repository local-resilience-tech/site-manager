import { Input, Textarea } from "@chakra-ui/react"
import { useForm } from "react-hook-form"

import { Field, FormActions, Button, FormFields } from "../../../components"

export interface NewRegionData {
  name: string
}

export type SubmitNewRegionFunc = (data: NewRegionData) => void

export default function NewRegion({
  onSubmitNewRegion,
}: {
  onSubmitNewRegion: SubmitNewRegionFunc
}) {
  const {
    register,
    handleSubmit,
    formState: { errors, isSubmitting },
  } = useForm<NewRegionData>()

  return (
    <form onSubmit={handleSubmit(onSubmitNewRegion)}>
      <FormFields>
        <Field
          label="Region Name"
          helperText={`A name to identify your Region - use lowercase letters and no spaces`}
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
      </FormFields>

      <FormActions>
        <Button loading={isSubmitting} type="submit">
          Create Region
        </Button>
      </FormActions>
    </form>
  )
}
