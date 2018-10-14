module Savings.Utils exposing (amount, encode, id, savingDecoder, savingUpdatedDecoder, savingsDecoder, title)

import Json.Decode
import Json.Decode.Pipeline
import Json.Encode as Encode exposing (Value, int, object, string)
import Savings.Models exposing (Saving)


savingsDecoder : Json.Decode.Decoder (List Saving)
savingsDecoder =
    Json.Decode.field "data" (Json.Decode.list savingDecoder)


savingDecoder : Json.Decode.Decoder Saving
savingDecoder =
    Json.Decode.succeed Saving
        |> Json.Decode.Pipeline.required "id" Json.Decode.int
        |> Json.Decode.Pipeline.required "title" Json.Decode.string
        |> Json.Decode.Pipeline.required "amount" Json.Decode.float


savingUpdatedDecoder : Json.Decode.Decoder Saving
savingUpdatedDecoder =
    Json.Decode.field "data" savingDecoder


id : Int -> ( String, Encode.Value )
id value =
    ( "id", Encode.int value )


title : String -> ( String, Encode.Value )
title value =
    ( "title", Encode.string value )


amount : Float -> ( String, Encode.Value )
amount value =
    ( "amount", Encode.float value )


encode : Saving -> Encode.Value
encode schema =
    Encode.object [ ( "saving", Encode.object [ id schema.id, title schema.title, amount schema.amount ] ) ]
