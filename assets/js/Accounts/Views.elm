module Accounts.Views exposing (adjustmentEditView, editView, renderAccount, renderAccounts, renderAdjustment)

import Accounts.Messages exposing (..)
import Accounts.Models exposing (..)
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import List


editView : Account -> Html Msg
editView model =
    div []
        [ input [ type_ "text", value model.name, onInput Accounts.Messages.NameUpdated ] []
        , input [ type_ "checkbox", checked model.debt, onClick Accounts.Messages.ToggleDebt ] []
        , input [ type_ "text", value (String.fromFloat model.balance), onInput Accounts.Messages.BalanceUpdated ] []
        , button [ onClick Accounts.Messages.SaveAccount, class "button" ] [ text "Save" ]
        ]


renderAccounts : List Account -> Html Msg
renderAccounts accounts =
    div []
        [ table []
            [ thead []
                [ tr []
                    [ th [] [ text "Name" ]
                    , th [] [ text "Balance" ]
                    , th [] [ text "Debt?" ]
                    , th [] [ text "Adjustments" ]
                    , th [] []
                    ]
                ]
            , tbody [] (List.concat (List.map renderAccount accounts))
            ]
        , button [ class "btn btn-primary", onClick CreateAccount ] [ text "New Account" ]
        ]


renderAccount : Account -> List (Html Msg)
renderAccount account =
    List.concat
        [ [ tr []
                [ td [] [ text account.name ]
                , td [] [ text (String.fromFloat account.balance) ]
                , td []
                    [ text
                        (if account.debt then
                            "Yes"

                         else
                            "No"
                        )
                    ]
                , td [ onClick (ToggleAdjustmentsFor account) ] [ text (account.adjustments |> adjustmentTotal |> String.fromFloat) ]
                , td []
                    [ span [ onClick (OpenAccountEditor account) ] [ text "Edit" ]
                    , span [ onClick (DeleteAccount account) ] [ text "Delete" ]
                    ]
                ]
          ]
        , renderAdjustmentsForAccount account
        ]


adjustmentTotal : List Adjustment -> Float
adjustmentTotal adjustments =
    List.foldl (+) 0.0 (List.map (\a -> a.total) adjustments)


renderAdjustmentsForAccount : Account -> List (Html Msg)
renderAdjustmentsForAccount account =
    case account.adjustmentsVisible of
        True ->
            List.map renderAdjustment account.adjustments

        False ->
            [ tr [] [ td [ colspan 5 ] [] ] ]


renderAdjustment : Adjustment -> Html Msg
renderAdjustment adjustment =
    tr []
        [ td [ colspan 3, onClick (OpenAdjustmentEditor adjustment) ] [ text adjustment.title ]
        , td []
            [ text (String.fromFloat adjustment.total) ]
        , td []
            [ span [ onClick (OpenAdjustmentEditor adjustment) ] [ text "Edit" ]
            , span [ onClick (DeleteAdjustment adjustment) ] [ text "Delete" ]
            ]
        ]


adjustmentEditView : Adjustment -> Html Msg
adjustmentEditView model =
    div []
        [ input [ type_ "text", value model.title, onInput TitleUpdated ] []
        , input [ type_ "text", value (String.fromFloat model.total), onInput TotalUpdated ] []
        , button [ onClick SaveAdjustment ] [ text "Save" ]
        ]
