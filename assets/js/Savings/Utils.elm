module Savings.Utils exposing (amount, encode, id, savingDecoder, savingUpdatedDecoder, savingsDecoder, title)

import Json.Decode
import Json.Decode.Pipeline
import Json.Encode
import Savings.Models


savingsDecoder : Json.Decode.Decoder (List Savings.Models.Saving)
savingsDecoder =
    Json.Decode.field "data" (Json.Decode.list savingDecoder)


savingDecoder : Json.Decode.Decoder Savings.Models.Saving
savingDecoder =
    Json.Decode.succeed Savings.Models.Saving
        |> Json.Decode.Pipeline.required "id" Json.Decode.int
        |> Json.Decode.Pipeline.required "title" Json.Decode.string
        |> Json.Decode.Pipeline.required "amount" Json.Decode.float


savingUpdatedDecoder : Json.Decode.Decoder Savings.Models.Saving
savingUpdatedDecoder =
    Json.Decode.field "data" savingDecoder


id : Int -> ( String, Json.Encode.Value )
id value =
    ( "id", Json.Encode.int value )


title : String -> ( String, Json.Encode.Value )
title value =
    ( "title", Json.Encode.string value )


amount : Float -> ( String, Json.Encode.Value )
amount value =
    ( "amount", Json.Encode.float value )


encode : Savings.Models.Saving -> Json.Encode.Value
encode schema =
    Json.Encode.object [ ( "saving", Json.Encode.object [ id schema.id, title schema.title, amount schema.amount ] ) ]
