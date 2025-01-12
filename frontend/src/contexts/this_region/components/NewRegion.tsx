import { Input, Textarea } from "@chakra-ui/react"
import { useForm, SubmitHandler } from "react-hook-form"

import { Field, FormActions, Button, FormFields } from "../../../components"

export interface NewRegionData {
  name: string
  description: string
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

        <Field
          label="Description"
          helperText={`A description of your Region`}
          invalid={!!errors.description}
          errorText={errors.description?.message}
        >
          <Textarea
            {...register("description", {
              maxLength: {
                value: 255,
                message: "Must be less than 255 characters",
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
