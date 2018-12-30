module Adjustments.Utils exposing (adjustmentDecoder, adjustmentUpdatedDecoder, adjustmentsDecoder, encode, id, title, total)

import Accounts.Models exposing (Adjustment)
import Json.Decode
import Json.Decode.Pipeline
import Json.Encode as Encode exposing (Value, int, object, string)


adjustmentsDecoder : Json.Decode.Decoder (List Adjustment)
adjustmentsDecoder =
    Json.Decode.field "data" (Json.Decode.list adjustmentDecoder)


adjustmentDecoder : Json.Decode.Decoder Adjustment
adjustmentDecoder =
    Json.Decode.succeed Adjustment
        |> Json.Decode.Pipeline.required "account_id" Json.Decode.int
        |> Json.Decode.Pipeline.required "id" Json.Decode.int
        |> Json.Decode.Pipeline.required "title" Json.Decode.string
        |> Json.Decode.Pipeline.required "total" Json.Decode.float


adjustmentUpdatedDecoder : Json.Decode.Decoder Adjustment
adjustmentUpdatedDecoder =
    Json.Decode.field "data" adjustmentDecoder


id : Int -> ( String, Encode.Value )
id value =
    ( "id", Encode.int value )


title : String -> ( String, Encode.Value )
title value =
    ( "title", Encode.string value )


total : Float -> ( String, Encode.Value )
total value =
    ( "total", Encode.float value )


encode : Adjustment -> Encode.Value
encode schema =
    Encode.object [ ( "adjustment", Encode.object [ id schema.id, title schema.title, total schema.total ] ) ]
