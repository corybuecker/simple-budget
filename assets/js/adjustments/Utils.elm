module Adjustments.Utils exposing (adjustmentDecoder, adjustmentUpdatedDecoder, adjustmentsDecoder, encode, id, title, total)

import Accounts.Models
import Json.Decode
import Json.Decode.Pipeline
import Json.Encode


adjustmentsDecoder : Json.Decode.Decoder (List Accounts.Models.Adjustment)
adjustmentsDecoder =
    Json.Decode.field "data" (Json.Decode.list adjustmentDecoder)


adjustmentDecoder : Json.Decode.Decoder Accounts.Models.Adjustment
adjustmentDecoder =
    Json.Decode.succeed Accounts.Models.Adjustment
        |> Json.Decode.Pipeline.required "account_id" Json.Decode.int
        |> Json.Decode.Pipeline.required "id" Json.Decode.int
        |> Json.Decode.Pipeline.required "title" Json.Decode.string
        |> Json.Decode.Pipeline.required "total" Json.Decode.float


adjustmentUpdatedDecoder : Json.Decode.Decoder Accounts.Models.Adjustment
adjustmentUpdatedDecoder =
    Json.Decode.field "data" adjustmentDecoder


id : Int -> ( String, Json.Encode.Value )
id value =
    ( "id", Json.Encode.int value )


title : String -> ( String, Json.Encode.Value )
title value =
    ( "title", Json.Encode.string value )


total : Float -> ( String, Json.Encode.Value )
total value =
    ( "total", Json.Encode.float value )


encode : Accounts.Models.Adjustment -> Json.Encode.Value
encode schema =
    Json.Encode.object [ ( "adjustment", Json.Encode.object [ id schema.id, title schema.title, total schema.total ] ) ]
